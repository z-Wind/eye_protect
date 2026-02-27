use clap::Parser;

/// GUI 共用參數（守護進程與 GUI binary 都使用相同欄位）
#[derive(Parser, Debug, Clone)]
pub struct GuiArgs {
    /// 啟用視窗置頂
    #[arg(short, long, help = "將護眼視窗固定在最上層，防止被其他視窗遮擋")]
    pub top_enable: bool,

    /// 倒數計時秒數
    #[arg(
        short,
        long,
        default_value_t = 20,
        value_parser = clap::value_parser!(u32).range(1..=3600),
        help = "設定護眼視窗出現的時間長度（單位：秒，範圍 1–3600）"
    )]
    pub wait_seconds: u32,

    /// 提醒文字內容
    #[arg(short, long, help = "在畫面中央顯示自定義的提醒訊息")]
    pub remind: Option<String>,
}
