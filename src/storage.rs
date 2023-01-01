use crate::eraftpb::{ConfState, Entry, HardState, Snapshot};
use crate::errors::Result;

#[derive(Debug, Clone)]
pub struct RaftState {
    pub hard_state: HardState,
    pub conf_state: ConfState,
}

pub trait Storage {

    /// initial_state方法返回RaftState的信息
    fn initial_state(&self) -> Result<RaftState>;

    /// entries returns a slice of log entries in the range [lo,hi).
    /// max_size limits the total size of the log entries returned, but
    /// entries returns at least one entry if any.
    fn entries(&self, low: u64, high: u64, max_size: u64) -> Result<Vec<Entry>>;

    /// term returns the term of entry idx, which must be in the range
    /// [first_index()-1, last_index()]. The term of the entry before
    /// first_index is retained for matching purpose even though the
    /// rest of that entry may not be available.
    fn term(&self, idx: u64) -> Result<u64>;

    /// first_index returns the index of the first log entry that is
    /// possible available via entries (older entries have been incorporated
    /// into the latest snapshot; if storage only contains the dummy entry the
    /// first log entry is not available).
    fn first_index(&self) -> Result<u64>;

    /// last_index返回log里最后一条entry的index
    fn last_index(&self) -> Result<u64>;

    /// snapshot方法返回最近一次的快照。如果快照暂时不可用, 则需要返回SnapshotTemporarilyUnavailable
    /// 这个错误, 从而让raft状态机知道storage需要一些时间准备快照，并在稍后再次调用本方法.
    fn snapshot(&self) -> Result<Snapshot>;
}
