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

/// 啟動 GUI 並獲取結束碼
/// - Some(0): 正常完成
/// - Some(1): 使用者跳過
/// - None: 啟動失敗或異常崩潰
fn launch_gui(exe: &PathBuf, gui_args: &[OsString]) -> Option<i32> {
    match Command::new(exe).args(gui_args).status() {
        Ok(status) => status.code(),
        Err(e) => {
            let cwd = env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "無法獲取".to_string());

            eprintln!(
                "[{}] 啟動 GUI 失敗!\n 錯誤: {e}\n 執行檔路徑: {}\n 當前工作目錄: {}",
                Local::now().format("%H:%M:%S"),
                exe.display(),
                cwd
            );

            None
        }
    }
}

pub fn main() {
    let args = Args::parse();

    if args.gui_mode {
        match gui::run(args.gui_args) {
            Ok(gui::ExitStatus::Completed) => std::process::exit(0),
            Ok(gui::ExitStatus::Skipped) | Ok(gui::ExitStatus::Aborted) => std::process::exit(1),
            Err(e) => {
                eprintln!("[{}] GUI 啟動崩潰: {e}", Local::now().format("%H:%M:%S"));
                std::process::exit(101); // 讓 Daemon 觸發異常計數
            }
        }
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

    let mut total_completed: u32 = 0;
    let mut total_skipped: u32 = 0;

    loop {
        println!("\n------------------------------------------------------------");
        println!(
            "[{}] 下次休息將在 {} 分鐘後...",
            Local::now().format("%H:%M:%S"),
            args.interval_minutes
        );

        let start_wait = std::time::Instant::now();
        thread::sleep(interval);

        // 如果實際睡眠時間遠超預期（例如超過預期的 1.5 倍），代表中間可能休眠了
        if start_wait.elapsed() > interval + Duration::from_secs(60) {
            println!(
                "[{}] 偵測到系統休眠或長時間停頓，跳過本次提醒並重新計時。",
                Local::now().format("%H:%M:%S")
            );
            continue;
        }

        println!(
            "[{}] 休息時間到！啟動護眼視窗...",
            Local::now().format("%H:%M:%S")
        );

        match launch_gui(&self_exe, &gui_args) {
            // 情況 A：明確的預期行為
            Some(0) => {
                consecutive_failures = 0;
                total_completed += 1;

                println!(
                    "[{}] 休息結束 [ 累計完成: {:>3} | 跳過: {:>3} ]",
                    Local::now().format("%H:%M:%S"),
                    total_completed,
                    total_skipped
                );
            }
            Some(1) => {
                consecutive_failures = 0;
                total_skipped += 1;

                println!(
                    "[{}] 休息跳過 [ 累計完成: {:>3} | 跳過: {:>3} ]",
                    Local::now().format("%H:%M:%S"),
                    total_completed,
                    total_skipped
                );
            }

            // 情況 B：非預期的 Exit Code (例如 101 Panic, 或是被 Task Manager 殺掉)
            // 情況 C：啟動失敗 (None)
            other => {
                consecutive_failures += 1;

                let error_msg = match other {
                    Some(code) => format!("非預期退出碼 ({})", code),
                    None => "無法啟動執行檔或進程被強制終止".to_string(),
                };

                if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    eprintln!(
                        "[{}] 錯誤: GUI 連續啟動失敗 {} 次 ({})，守護進程退出。",
                        Local::now().format("%H:%M:%S"),
                        MAX_CONSECUTIVE_FAILURES,
                        error_msg
                    );
                    std::process::exit(1);
                }

                eprintln!(
                    "[{}] 警告: GUI 啟動失敗（{}/{}），將在下個週期重試。原因: {}",
                    Local::now().format("%H:%M:%S"),
                    consecutive_failures,
                    MAX_CONSECUTIVE_FAILURES,
                    error_msg
                );
            }
        }
    }
}
