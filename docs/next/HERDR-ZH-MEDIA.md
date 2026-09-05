# herdr-zh：工作區圖片與文件

## 使用方式

1. 點工作區列表的新增按鈕，輸入名稱、專案資料夾完整路徑。預設為 `~/Herdr Projects/project-N`；修改名稱會同步預設路徑，也可自行修改完整路徑。
2. 新專案會建立專案目錄和 `media/`。要綁定現有專案，勾選「開啟既有資料夾」。既有檔案不會被覆蓋。
3. 在工作區終端啟動 Codex 或 Claude。代理收到 `HERDR_MEDIA_DIR`，專案 `AGENTS.md` 和 `CLAUDE.md` 會加入有標記的輸出指引；既有內容保留。建議每次任務使用不同子資料夾。
4. 右側「設定」可修改媒體目錄，選擇是否複製既有媒體、是否收集專案內新增或更新的成品。設定保存在專案 `.herdr/media.json`，重新開啟工作區仍生效。改變路徑後重新啟動代理，更新啟動時的環境變數。
5. 拖入檔案到代理終端或貼上截圖：檔案存入媒體資料夾，副本路徑交給代理，原檔保留。一般文字和指令不改寫。拖入辨識需終端傳送貼上事件。
6. 點檔案使用 macOS Quick Look 預覽，右鍵使用預設程式開啟；捲輪瀏覽清單，點「全部／圖片／文件」切換篩選。可開啟 Finder 或重新掃描。導覽模式 `m` 收合面板、`M` 開設定；選取檔案後空白鍵預覽。

支援圖片：JPG/JPEG、PNG、GIF、WebP、HEIC。
支援文件：PDF、Word (DOC/DOCX)、Excel (XLS/XLSX/CSV)、PowerPoint (PPT/PPTX)、TXT、Markdown。
Quick Look 是否能呈現特定文件取決於 macOS 的預覽支援；可改用預設程式開啟。預覽視窗在執行 Herdr server 的本機開啟。

## 收集行為

- `media/`（或設定目錄）是真實檔案來源，初次開啟即掃描，約每秒核對新增、修改、刪除；手動放入、關閉程式期間新增的媒體也會出現。
- 新建專案預設啟用成品收集；既有專案預設關閉，避免自動收集既有程式素材。
- 收集啟用後，專案目錄內新建或更新的常用圖片／文件，連續兩次掃描大小、修改時間相同後複製到 `media/collected/`。原檔永不搬移。這是寫入穩定的判斷，代理若長時間暫停寫入，可能保留中間版本；最终版本會另外收集。最可靠的方法仍是直接產出到媒體目錄。
- 程式碼、影音、隱藏檔、依賴／build 目錄與 AGENTS/CLAUDE/README 等指引文件不自動收集。掃描不追蹤符號連結，最多 32 層。
- 自動收集只涵蓋監看啟動後的變動；專案外的成品需匯入。AI 是否遵守輸出指引仍取決於代理，Herdr 不會改變其權限設定。
- 匯入副本以內容雜湊去重並保留不同版本，避免同名覆寫。清單省略雜湊顯示原檔名。
- 改變媒體目錄可複製舊媒體，原目錄仍保留。媒體目錄不得是專案本身或專案的祖先目錄。

## 維護與合併上游

本功能集中在 `src/media/`（收集/設定/掃描）、`src/app/media.rs`（操作）、`src/ui/media_panel.rs`（顯示），只在既有工作區、貼上和繪圖入口接線。JSON API 增加 `project.create`、`workspace.media.get`、`workspace.media.set`；不改既有 session identity 或 binary wire 格式。沒有額外第三方依賴。

更新方式為合併上游原始碼後重建，不能以官方 `herdr update` 下載的執行檔替代 herdr-zh，否則會失去 fork 功能。採單一 fork 分支和少量獨立功能模組，不另做更新框架。

在目前修改完成並提交後，維護者可依序：

```sh
# 第一次設定一次即可
git remote add upstream https://github.com/herdrdev/herdr.git
# 更新時在獨立分支檢視與合併；保留 fork 的 README/Cargo metadata/繁中內容
git fetch upstream
git switch -c maintenance/upstream-sync
git merge upstream/master
just check
just build
```

若 upstream 的工作區、輸入或 render 入口變動，調整接線並跑媒體回歸测试。Git 不保證每次無衝突；先驗證後再合回 fork 主分支。

本機 Homebrew 工具鏈範例（此專案 Rust 1.96.1 / Zig 0.15.2）：

```sh
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
export ZIG=/opt/homebrew/opt/zig@0.15/bin/zig
just ci 'test(media) | test(new_workspace)'
```
