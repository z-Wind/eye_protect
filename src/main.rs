use chrono::Local;
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "簡單高效的跨平台護眼提醒守護程序",
    long_about = "本程式會定期啟動護眼視窗，強制提醒您休息。支援全螢幕顯示與置頂功能。"
)]
struct Args {
    /// 是否開啟視窗置頂功能 (Always on Top)
    #[arg(short, long, help = "開啟此選項後，護眼視窗將會覆蓋在所有視窗之上")]
    top_enable: bool,

    /// 每次休息的持續秒數
    #[arg(
        short,
        long,
        default_value_t = 20,
        help = "設定護眼視窗出現的時間長度（單位：秒）"
    )]
    wait_seconds: i32,

    /// 兩次休息之間的時間間隔（分鐘）
    #[arg(
        short,
        long,
        default_value_t = 10,
        help = "設定每隔多久提醒一次休息（單位：分鐘）"
    )]
    interval_minutes: u64,

    /// 在護眼視窗上顯示的提醒文字
    #[arg(
        short,
        long,
        help = "設定自定義的提醒文字（例如：喝口水、站起來動一動）"
    )]
    remind: Option<String>,
}

fn get_main_executable() -> PathBuf {
    let exe = env::current_exe().expect("無法獲取當前執行檔路徑");
    let dir = exe.parent().expect("無法獲取目錄");

    // 使用 std::env::consts::EXE_EXTENSION 自動處理 .exe 擴充名
    let mut main_path = dir.join("main");
    main_path.set_extension(env::consts::EXE_EXTENSION);
    main_path
}

pub fn main() {
    let args = Args::parse();
    let main_path = get_main_executable();

    // 打印運行資訊
    println!("--- 護眼守護進程已啟動 ---");
    println!("目標執行檔: {}", main_path.display());
    println!("作業系統: {}", env::consts::OS);
    println!(
        "設定: 每 {} 分鐘休息 {} 秒",
        args.interval_minutes, args.wait_seconds
    );
    if args.top_enable {
        println!("已開啟 Always-on-top");
    }

    // 檢查文件是否存在
    if !main_path.exists() {
        panic!(
            "錯誤: 找不到目標程式 {:?}，請確保它與守護程序在同一目錄下。",
            main_path
        );
    }

    // 預先構建 Command 範本（避免在 loop 中重複判斷 args）
    let mut command = Command::new(&main_path);
    if args.top_enable {
        command.arg("-t");
    }
    command.args(["-w", &args.wait_seconds.to_string()]);
    if let Some(remind) = &args.remind {
        command.args(["-r", remind]);
    }

    loop {
        // 等待間隔
        thread::sleep(time::Duration::from_secs(60 * args.interval_minutes));

        println!(
            "[{}] 休息時間到！啟動護眼視窗...",
            Local::now().format("%H:%M:%S")
        );

        // 啟動並等待 GUI 結束
        match command.status() {
            Ok(status) => println!(
                "[{}] 休息結束 (結束碼: {})",
                Local::now().format("%H:%M:%S"),
                status
            ),
            Err(e) => eprintln!("  啟動 GUI 失敗: {}", e),
        }
    }
}
