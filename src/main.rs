use chrono::Local;
use clap::Parser;
use eye_protect::GuiArgs;
use std::{env, path::PathBuf, process::Command, thread, time::Duration};

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

    #[command(flatten)]
    gui: GuiArgs,
}

/// 取得 GUI 執行檔路徑。
/// 假設 `eye_protect_gui` 與本執行檔位於同一目錄。
/// 若打包方式改變，請確保兩個執行檔維持在相同目錄下。
fn get_gui_executable() -> PathBuf {
    let exe = env::current_exe().expect("無法獲取當前執行檔路徑");
    let dir = exe.parent().expect("無法獲取執行檔所在目錄");
    let mut path = dir.join("eye_protect_gui");
    path.set_extension(env::consts::EXE_EXTENSION);
    path
}

fn build_gui_args(gui: &GuiArgs) -> Vec<String> {
    let mut args = vec!["-w".to_string(), gui.wait_seconds.to_string()];
    if gui.top_enable {
        args.push("-t".to_string());
    }
    if let Some(remind) = &gui.remind {
        args.push("-r".to_string());
        args.push(remind.clone());
    }
    args
}

/// 啟動 GUI 並等待結束。
/// 回傳 `true` 表示成功啟動（不論使用者是否提前離開），`false` 表示啟動失敗。
fn launch_gui(exe: &PathBuf, gui_args: &[String]) -> bool {
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
    let gui_exe = get_gui_executable();

    println!("--- 護眼守護進程已啟動 ---");
    println!("目標執行檔: {}", gui_exe.display());
    println!("作業系統: {}", env::consts::OS);
    println!(
        "設定: 每 {} 分鐘休息 {} 秒",
        args.interval_minutes, args.gui.wait_seconds
    );
    if args.gui.top_enable {
        println!("已開啟 Always-on-top");
    }

    if !gui_exe.exists() {
        eprintln!(
            "錯誤: 找不到 {:?}。\n\
             請確保 eye_protect_gui 與本執行檔位於同一目錄下。",
            gui_exe
        );
        std::process::exit(1);
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

    let gui_args = build_gui_args(&args.gui);
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

        if launch_gui(&gui_exe, &gui_args) {
            consecutive_failures = 0;
        } else {
            consecutive_failures += 1;
            if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                eprintln!(
                    "錯誤: GUI 連續啟動失敗 {} 次，守護進程退出。\n\
                     請確認 {:?} 是否存在且有執行權限。",
                    MAX_CONSECUTIVE_FAILURES, gui_exe
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
