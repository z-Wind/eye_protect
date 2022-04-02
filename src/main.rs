use chrono::Local;
use std::process::Command;
use std::{thread, time};

pub fn main() {
    loop {
        thread::sleep(time::Duration::from_secs(60 * 10));
        println!("start: {}", Local::now().format("%H:%M:%S"));
        let mut protect = Command::new("./main.exe")
            .spawn()
            .expect("failed to execute main.exe");

        let _ecode = protect.wait().expect("failed to wait");
        println!("  end: {}", Local::now().format("%H:%M:%S"));
    }
}
