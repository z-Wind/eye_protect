use chrono::Local;
use clap::Parser;
use std::env;
use std::process::Command;
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    top_enable: bool,
    #[arg(short, long, default_value_t = 20)]
    wait_seconds: i32,
    /// sleep every X minutes
    #[arg(short, long, default_value_t = 10)]
    interval_minutes: u64,
    /// Print to screen, only support ascii code
    #[arg(short, long)]
    remind: Option<String>,
}

pub fn main() {
    let args = Args::parse();
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
    println!("OS:{}", env::consts::OS);
    println!("execute at {}", Local::now().format("%H:%M:%S"));
    println!("alway on top: {}", args.top_enable);
    println!(
        "wait for {} seconds every {} minutes",
        args.wait_seconds, args.interval_minutes
    );
    assert!(main_path.exists());

    let mut command = Command::new(&main_path);
    if args.top_enable {
        command.arg("-t");
    }
    command.args(["-w", &format!("{}", args.wait_seconds)]);
    if let Some(remind) = args.remind.as_deref() {
        command.args(["-r", remind]);
    }
    // println!("{:?}", command);

    loop {
        thread::sleep(time::Duration::from_secs(60 * args.interval_minutes));
        println!("start: {}", Local::now().format("%H:%M:%S"));
        let mut protect = command.spawn().expect("failed to execute main.exe");
        let _ecode = protect.wait().expect("failed to wait");
        println!("  end: {}", Local::now().format("%H:%M:%S"));
    }
}
