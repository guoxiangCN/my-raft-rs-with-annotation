use crate::eraftpb::{Entry, Snapshot};

/// unstable.entries[i]的日志为(unstable.offset + i)索引位置上的.
/// 注意unstable.offset可能比storage里最高的索引位置要小,这意味
/// 着下一次往storage的写入需要truncate日志(在持久化unsstable.entries之前)
#[derive(Debug, PartialEq, Default)]
pub struct Unstable {
    // 刚收到的Snapshot(如果有)
    pub snapshot: Option<Snapshot>,

    // 所有还没被写入storage的entries
    pub entries: Vec<Entry>,
    pub offset: u64,

    pub tag: String,
}

impl Unstable {
    pub fn new(offset: u64, tag: String) -> Unstable {
        Unstable {
            snapshot: None,
            entries: vec![],
            offset: offset,
            tag: tag,
        }
    }

    /// maybe_first_index返回entries中的第一个可能的entry的索引
    /// (前提是有snapshot存在,返回值为snapshot的index+1)
    pub fn maybe_first_index(&self) -> Option<u64> {
        self.snapshot
            .as_ref()
            .map(|snap| snap.get_metadata().get_index() + 1)
    }

    pub fn maybe_last_index(&self) ->Option<u64> {
        match self.entries.len() {
            0 => self.snapshot.as_ref().map(|snap|snap.get_metadata().get_index()),
            len => Some(self.offset+len as u64 - 1),
        }
    }

    /// maybe_term返回给定index位置的日志条目的任期(如果该idx存在)
    pub fn maybe_term(&self, idx: u64) -> Option<u64> {
        if idx < self.offset {
            if self.snapshot.is_none() {
                return None;
            }
            let meta = self.snapshot.as_ref().unwrap().get_metadata();
            if idx == meta.get_index() {
                return Some(meta.get_term());
            }
            return None;
        }
        self.maybe_last_index().and_then(|last|{
            if idx > last {
                return None;
            }
            Some(self.entries[(idx-self.offset) as usize].get_term())
        })
    }
}
