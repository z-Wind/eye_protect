# eye_protect

簡單高效的跨平台護眼提醒工具。定時彈出全螢幕倒數視窗，強制讓眼睛休息。

## 功能

- 可設定休息間隔與休息時長
- 全螢幕黑底倒數，低飽和護眼配色
- 支援自訂提醒文字
- 支援視窗置頂（Always on Top）
- 倒數顏色隨剩餘時間漸變（綠白 → 琥珀 → 霧橘）
- 按 ESC 可提前關閉休息視窗
- Ctrl+C 優雅停止守護進程

## 安裝

需要先安裝 [Rust](https://rust-lang.org/)，然後執行：

```sh
cargo build --release
```

編譯後會在 `target/release/` 產生兩個執行檔：

| 執行檔 | 說明 |
|---|---|
| `eye_protect` | 守護進程，負責定時觸發休息 |
| `eye_protect_gui` | GUI 視窗，由守護進程自動呼叫 |

> **注意**：兩個執行檔必須放在同一目錄下才能正常運作。

## 使用方式

啟動守護進程即可，預設每 10 分鐘提醒一次，每次休息 20 秒：

```sh
eye_protect
```

### 參數說明

```sh
eye_protect [OPTIONS]

Options:
  -i, --interval-minutes <N>  休息間隔（分鐘，範圍 1–720，預設 10）
  -w, --wait-seconds <N>      每次休息秒數（範圍 1–3600，預設 20）
  -r, --remind <TEXT>         自訂提醒文字
  -t, --top-enable            開啟視窗置頂
  -h, --help                  顯示說明
  -V, --version               顯示版本
```

### 範例

```sh
# 每 25 分鐘休息 20 秒（20-20-20 護眼法則）
eye_protect -i 25 -w 20

# 每 30 分鐘休息，顯示自訂提醒文字，並開啟置頂
eye_protect -i 30 -w 30 -r "站起來動一動！" -t
```

## 專案結構

```
src/
├── lib.rs          # 共用參數定義（GuiArgs）
├── main.rs         # 守護進程
└── bin/
    └── gui.rs      # GUI 視窗
```

## License

MIT
