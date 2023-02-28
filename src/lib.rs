

pub mod eraftpb;
pub mod storage;
pub mod errors;
pub mod status;

mod read_only;
mod log_unstable;
mod util;
mod progress;

#[cfg(test)]
mod tests {

    use std::sync::mpsc;

    #[test]
    fn test_std_sync_channel() {
        let (sender, receiver) = mpsc::sync_channel::<String>(4);
        sender.send(String::from("value1")).unwrap();
        sender.send(String::from("value2")).unwrap();
        sender.send(String::from("value3")).unwrap();
        sender.send(String::from("value4")).unwrap();
        println!("push 4 not block....");
        // sender.send("value5".to_owned()).unwrap(); // will block
    }

    #[test]
    fn test_std_channel() {
        // super max buffer, and never block the sender.
        let (sender, receiver) = mpsc::channel::<String>();
        sender.send(String::from("value1")).unwrap();
        sender.send(String::from("value2")).unwrap();
        sender.send(String::from("value3")).unwrap();
        sender.send(String::from("value4")).unwrap();
        loop {
            match receiver.recv_timeout(std::time::Duration::from_secs(3)){
                Ok(x) => println!("{}",x),
                Err(e) => {
                    println!("error = {}", e.to_string()); // timed out waiting on channel
                    break;
                },
            }   
        }
    }

}