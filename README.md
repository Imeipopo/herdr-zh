# herdr-zh

<p align="center">
  <img src="assets/logo.png" alt="herdr-zh" width="100" />
</p>

<p align="center">
  Herdr 的非官方繁體中文版本
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-666666?labelColor=333333" alt="Apache 2.0 license" /></a>
  <a href="https://github.com/herdrdev/herdr"><img src="https://img.shields.io/badge/upstream-herdrdev%2Fherdr-666666?labelColor=333333&logo=github" alt="upstream herdrdev/herdr" /></a>
</p>

> [!IMPORTANT]
> `herdr-zh` 是由社群維護的非官方衍生版本，並非 Herdr 官方發行版。原始專案為 [herdrdev/herdr](https://github.com/herdrdev/herdr)，本專案保留其 [Apache License 2.0](LICENSE) 授權與完整 Git 歷史。

Herdr 是在終端機中管理 AI 程式代理的工作空間工具。`herdr-zh` 在保留原有功能的基礎上，加入繁體中文介面與可切換的安全自動模式。

## 這個版本增加了什麼

- 主要介面、設定、全域選單、右鍵選單與操作對話框繁體中文化。
- 主題選單加入繁中功能名稱與「深色／淺色」標示。
- 介面顯示並可切換「自動模式：開／關」。
- Herdr 啟動或恢復 Codex 時，安全自動模式使用 `workspace-write` 與 `on-request`。
- Herdr 啟動或恢復 Claude Code 時，安全自動模式使用 `permission-mode auto`。
- 保留英文介面，可在設定檔切換語言。

安全自動模式不會使用跳過權限檢查的選項，也不會影響已在執行中的代理，或你在窗格內手動輸入的 `codex`／`claude` 指令。

## 建置

需要 Rust 與專案指定的工具鏈。完整依賴與平台說明請參考[上游文件](https://herdr.dev/docs/)。

```bash
git clone https://github.com/Imeipopo/herdr-zh.git
cd herdr-zh
cargo build --release
./target/release/herdr
```

若電腦已安裝官方版 Herdr，建議先直接執行本專案的建置產物，避免覆蓋原本的 `herdr` 指令。

## 繁中與自動模式設定

在 Herdr 設定檔加入：

```toml
[ui]
language = "zh-TW"
auto_mode = true
media_panel_start_collapsed = false
```

也可以在繁中介面的「設定 → 自動模式」隨時切換，變更會保存到設定檔。

## Herdr 原有功能

- 同時查看各個代理的工作、等待與完成狀態。
- 分離終端後代理持續執行，稍後可重新連線。
- 支援鍵盤、滑鼠、分割窗格、遠端連線、外掛與 socket API。
- 單一 Rust 執行檔，不依賴 Electron。

使用方式與完整功能請閱讀 [Herdr 官方文件](https://herdr.dev/docs/)。上游原始碼、問題回報與官方版本位於 [herdrdev/herdr](https://github.com/herdrdev/herdr)。

## 來源與授權

本專案衍生自：

- 專案：Herdr
- 上游原始碼：[https://github.com/herdrdev/herdr](https://github.com/herdrdev/herdr)
- 建立本繁中版本時的基準提交：`1997b88b3fa45f838d44e69dcebde8acf33899fc`
- 上游授權：[Apache License 2.0](LICENSE)

繁中介面與安全自動模式是本衍生版本的修改。更完整的來源聲明請見 [NOTICE](NOTICE)。Herdr 名稱、標誌與上游內容的權利歸其各自權利人所有；本專案不代表 Herdr 官方背書。
