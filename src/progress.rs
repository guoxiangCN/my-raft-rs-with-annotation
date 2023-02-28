use ::flat_map::FlatMap;
use ::flat_map::flat_map;
use std::iter::Chain;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProgressState {
    // 探测状态. 当follower拒绝了最近的append消息时,则会进入探测状态, 此时Leader会视图试图往前
    // 追溯该follower的日志从何处开始丢失。在Probe状态, leader每次最多append一条日志,如果收到
    // 的回应中带有RejectHint信息，则回退Next索引以便下次重试。
    // 在初始时, Leader会把所有follower节点的状态都设置为Probe, 因此它并不知道各个follower的同步
    // 状态, 需要慢慢试探.
    Probe,

    // 正常复制状态. 如果试探到了具体的位置则会切换状态并以pipeline的方式快速复制日志。Leader在
    // 发送复制消息之后就修改该节点的Next索引为发送消息的最大索引+1.
    Replicate,

    // 接受快照状态. 当Ledaer向某个follower发送Append消息，试图让该follower状态跟上Leader时，若发现
    // 此时Leader上保存的索引数据已经对不上，比如Leader在Index=10之前的数据都已经写入快照中了, 但是
    // 该Follower需要的是10之前的数据, 这种情况下则会切换到Snapshot状态，然后发送快照给follower.
    // 当快照数据同步之后, 并不是直接切到Replicate状态，而是首先切换到Probe状态.
    Snapshot,
}

impl Default for ProgressState {
    fn default() -> Self {
        ProgressState::Probe
    }
}

pub struct ProgressSet {
    voters: FlatMap<u64, Progress>,
    learners: FlatMap<u64, Progress>,
}

impl ProgressSet {
    pub fn new(voter_size: usize, leader_size: usize) -> Self {
        ProgressSet {
            voters: FlatMap::with_capacity(voter_size),
            learners: FlatMap::with_capacity(leader_size),
        }
    }

    #[inline]
    pub fn voters(&self) -> &FlatMap<u64, Progress> {
        &self.voters
    }

    #[inline]
    pub fn learners(&self) -> &FlatMap<u64, Progress> {
        &self.learners
    }

    /// Only returns the voters, the learners are ignored.
    pub fn nodes(&self) -> Vec<u64> {
        let mut nodes = Vec::with_capacity(self.voters.len());
        nodes.extend(self.voters.keys());
        nodes.sort();
        nodes
    }

    pub fn learner_notes(&self) -> Vec<u64> {
         let mut ids = Vec::with_capacity(self.learners.len());
        ids.extend(self.learners.keys());
        ids.sort();
        ids
    }

    pub fn get(&self, id: u64) -> Option<&Progress> {
        self.voters.get(&id).or_else(||self.learners.get(&id))
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Progress> {
        let progress = self.voters.get_mut(&id);
        if progress.is_none() {
            return self.learners.get_mut(&id);
        }
        return progress;
    }

    pub fn iter(&self) -> Chain<flat_map::Iter<u64,Progress>, flat_map::Iter<u64, Progress>> {
        self.voters.iter().chain(self.learners.iter())
    }

    pub fn iter_mut(
        &mut self,
    ) -> Chain<flat_map::IterMut<u64, Progress>, flat_map::IterMut<u64, Progress>> {
        self.voters.iter_mut().chain(&mut self.learners)
    }

    pub fn insert_voter(&mut self, id: u64, pr: Progress) {
        if self.learners.contains_key(&id) {
            panic!("insert voter {} but already in learners", id);
        }
        if self.voters.insert(id, pr).is_some() {
            panic!("insert voter {} twice", id);
        }
    }

    pub fn insert_learner(&mut self, id:u64, pr: Progress) {
        if self.voters.contains_key(&id) {
            panic!("insert learner {} but already in voters", id);
        }
        if self.learners.insert(id, pr).is_some() {
            panic!("insert learner {} twice", id);
        }
    }

    pub fn remove(&mut self, id: u64) -> Option<Progress> {
        match self.voters.remove(&id) {
            None => self.learners.remove(&id),
            Some(x) => Some(x),
        }
    }

    pub fn promote_learner(&mut self, id: u64) {
        if let Some(mut pr) = self.learners.remove(&id) {
            pr.is_learner = false;
            self.voters.insert(id, pr);
            return;
        }
        panic!("promote a not existed learner: {}", id)
    }

}

#[derive(Debug,Default,Clone)]
pub struct Progress {
    /// raft论文里的matchIndex, 代表针对该follower节点，已知的的和Leader匹配的最高日志索引.
    /// 即follower的lastApplied.
    pub matched: u64,

    /// raft论文里的nextIndex, 代表针对该follower节点, 下一个需要发送的日志条目索引.
    /// 初始化为Leader的最后一个日志索引+1
    pub next_idx: u64,

    /// 当前节点的同步进度状态.
    pub state: ProgressState,

    /// Paused作用于ProgressState::Probe时.
    /// 当Paused=true, raft需要暂停发送ReplicationMessage到该节点.
    pub paused: bool,

    /// pending_snapshot作用于ProgressState::Snapshot时.
    /// TODO
    pub pending_snapshot: u64,

    /// 当从相应的follower收到任何消息都代表progress是活跃的.
    /// 在经过election_timeout之后,recent_active会被设置成false
    pub recent_active: bool,

    /// 一个滑动窗口用于控制发送给follower的消息
    pub ins: Inflights,

    /// 是否是Learner节点.
    pub is_learner: bool,
}

impl Progress {

    fn reset_state(&mut self, state: ProgressState) {
        self.paused=false;
        self.pending_snapshot=0;
        self.state=state;
        self.ins.reset();
    }

    pub fn become_probe(&mut self) {
        // 如果原来的状态是Snapshot,则代表Progress知道pending的snapshot已经成功
        // 发送给了这个Peer, 那么则可以直接从pending_snapshot+1开始探测.
        if self.state  == ProgressState::Snapshot {
            let pending_snapshot = self.pending_snapshot;
            self.reset_state(ProgressState::Probe);
            self.next_idx=std::cmp::max(self.matched+1, pending_snapshot+1);
        } else {
            self.reset_state(ProgressState::Probe);
            self.next_idx = self.matched + 1;
        }
    }

    pub fn become_replicate(&mut self) {
        self.reset_state(ProgressState::Replicate);
        self.next_idx = self.matched + 1;
    }

    pub fn become_snapshot(&mut self, snapshot_idx: u64) {
        self.reset_state(ProgressState::Snapshot);
        self.pending_snapshot = snapshot_idx;
    }

    #[inline]
    pub fn snapshot_failure(&mut self) {
        self.pending_snapshot=0;
    }

    #[inline]
    pub fn maybe_snapshot_abort(&self) -> bool {
        self.state==ProgressState::Snapshot && self.matched >= self.pending_snapshot
    }

    pub fn maybe_update(&mut self, n: u64) -> bool {
        let need_update = self.matched < n;
        if need_update {
            self.matched = n;
            self.resume();
        }

        if self.next_idx < n+1 {
            self.next_idx = n+1
        }

        need_update
    }

    pub fn optimistic_update(&mut self, n: u64) {
        self.next_idx = n+1;
    }

    #[inline]
    pub fn is_paused(&self)->bool {
        match self.state {
            ProgressState::Probe => self.paused,
            ProgressState::Replicate => self.ins.full(),
            ProgressState::Snapshot=>true,
        }
    }

    pub fn pause(&mut self) {
        self.paused=true;
    }

    pub fn resume(&mut self) {
        self.paused=false;
    }

}

#[derive(Debug,Default,Clone)]
pub struct Inflights {

}

impl Inflights {
    fn reset(&mut self) {
        // TODO
    }

    fn full(&self) -> bool {
        false
    }
}