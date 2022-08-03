use std::thread;

pub fn sleep_100_millis() {
    thread::sleep(std::time::Duration::from_millis(100));
}
