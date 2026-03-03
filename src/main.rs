mod gui;

use chrono::Local;
use clap::Parser;
use gui::GuiArgs;
use std::{env, ffi::OsString, path::PathBuf, process::Command, thread, time::Duration};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "簡單高效的跨平台護眼提醒守護程序",
    long_about = "本程式會定期啟動護眼視窗，強制提醒您休息。支援全螢幕顯示與置頂功能。"
)]
struct Args {
    /// 兩次休息之間的時間間隔（分鐘）
    #[arg(
        short,
        long,
        default_value_t = 10,
        value_parser = clap::value_parser!(u64).range(1..=720),
        help = "設定每隔多久提醒一次休息（單位：分鐘，範圍 1–720）"
    )]
    interval_minutes: u64,

    /// 隱藏的 GUI 模式開關
    #[arg(long, hide = true)]
    gui_mode: bool,

    #[command(flatten)]
    gui_args: GuiArgs,
}

/// 啟動 GUI 並等待結束。
/// 回傳 `true` 表示成功啟動（不論使用者是否提前離開），`false` 表示啟動失敗。
fn launch_gui(exe: &PathBuf, gui_args: &[OsString]) -> bool {
    match Command::new(exe).args(gui_args).status() {
        Ok(status) => {
            println!(
                "[{}] 休息結束 (結束碼: {})",
                Local::now().format("%H:%M:%S"),
                status
            );
            true
        }
        Err(e) => {
            eprintln!("[{}] 啟動 GUI 失敗: {e}", Local::now().format("%H:%M:%S"));
            false
        }
    }
}

pub fn main() {
    let args = Args::parse();

    if args.gui_mode {
        if let Err(e) = gui::run(args.gui_args) {
            eprintln!("[{}] 啟動 GUI 失敗: {e}", Local::now().format("%H:%M:%S"));
        };
    } else {
        run_daemen(args)
    }
}

fn run_daemen(args: Args) {
    let self_exe = env::current_exe().expect("無法獲取當前執行檔路徑");

    println!("--- 護眼守護進程已啟動 ---");
    println!("目標執行檔: {}", self_exe.display());
    println!("作業系統: {}", env::consts::OS);
    println!(
        "設定: 每 {} 分鐘休息 {} 秒",
        args.interval_minutes, args.gui_args.wait_seconds
    );
    if args.gui_args.top_enable {
        println!("已開啟 Always-on-top");
    }

    // #4 攔截 Ctrl+C，印出提示後優雅退出
    ctrlc::set_handler(|| {
        println!(
            "\n[{}] 收到中斷信號，守護進程已停止。",
            Local::now().format("%H:%M:%S")
        );
        std::process::exit(0);
    })
    .expect("無法設定 Ctrl+C 處理器");

    // 獲取目前進程啟動時的所有原始參數
    // 過濾掉重複的 gui-mode 防止參數污染
    let mut gui_args: Vec<OsString> = env::args_os()
        .skip(1)
        .filter(|arg| arg != "--gui-mode")
        .collect();
    gui_args.push(OsString::from("--gui-mode"));

    let interval = Duration::from_secs(60 * args.interval_minutes);

    // #6 連續失敗計數，超過門檻自動退出
    const MAX_CONSECUTIVE_FAILURES: u32 = 3;
    let mut consecutive_failures: u32 = 0;

    loop {
        println!(
            "[{}] 下次休息將在 {} 分鐘後...",
            Local::now().format("%H:%M:%S"),
            args.interval_minutes
        );
        thread::sleep(interval);
        println!(
            "[{}] 休息時間到！啟動護眼視窗...",
            Local::now().format("%H:%M:%S")
        );

        if launch_gui(&self_exe, &gui_args) {
            consecutive_failures = 0;
        } else {
            consecutive_failures += 1;
            if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                eprintln!(
                    "錯誤: GUI 連續啟動失敗 {} 次，守護進程退出。\n\
                     請確認 {:?} 是否存在且有執行權限。",
                    MAX_CONSECUTIVE_FAILURES, self_exe
                );
                std::process::exit(1);
            }
            eprintln!(
                "警告: GUI 啟動失敗（{}/{}），將在下個週期重試。",
                consecutive_failures, MAX_CONSECUTIVE_FAILURES
            );
        }
    }
}
