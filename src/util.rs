use protobuf::Message;
use std::u64;

pub const NO_LIMIT: u64 = u64::MAX;

pub fn limit_size<T: Message + Clone>(entries: &mut Vec<T>, max: u64) {
    if max == NO_LIMIT || entries.len() <= 1 {
        return;
    }

    let mut size = 0;
    let limit = entries
        .iter()
        .take_while(|e| {
            // At least 1 entry will be returned.
            if size == 0 {
                size += e.compute_size() as u64;
                true
            } else {
                size += e.compute_size() as u64;
                size <= max
            }
        })
        .count();
    entries.truncate(limit);
}
