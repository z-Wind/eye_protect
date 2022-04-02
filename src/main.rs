use chrono::Local;
use std::env;
use std::process::Command;
use std::{thread, time};

pub fn main() {
    println!("OS:{}", env::consts::OS);

    let exe = env::current_exe().expect("A path");
    let dir = exe.parent().expect("Executable must be in some directory");

    let main_path = if env::consts::OS == "linux" {
        dir.join("main")
    } else if env::consts::OS == "windows" {
        dir.join("main.exe")
    } else {
        panic!("{} not support", env::consts::OS);
    };
    println!("main_path:{}", main_path.display());
    assert!(main_path.exists());

    loop {
        thread::sleep(time::Duration::from_secs(60 * 10));
        println!("start: {}", Local::now().format("%H:%M:%S"));
        let mut protect = Command::new(&main_path)
            .spawn()
            .expect("failed to execute main.exe");

        let _ecode = protect.wait().expect("failed to wait");
        println!("  end: {}", Local::now().format("%H:%M:%S"));
    }
}
