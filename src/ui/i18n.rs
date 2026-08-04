use crate::config::UiLanguageConfig;

pub(crate) fn tr(language: UiLanguageConfig, text: &'static str) -> &'static str {
    if language == UiLanguageConfig::English {
        return text;
    }

    match text {
        "settings" => "設定",
        "theme" => "主題",
        "indicators" => "狀態指示",
        "auto mode" => "自動模式",
        "sound" => "音效",
        "toasts" => "通知",
        "pane labels" => "窗格標籤",
        "integrations" => "代理整合",
        "agent status indicators" => "代理狀態指示",
        "choose color dots or distinct symbols for each state" => {
            "選擇彩色圓點或不同符號來顯示各種狀態"
        }
        "color dots  ● ● ● ○ ·" => "彩色圓點  ● ● ● ○ ·",
        "distinct symbols  × ◐ ✓ ○ ·" => "不同符號  × ◐ ✓ ○ ·",
        "automatic mode" => "自動模式",
        "use safe auto permissions for codex and claude agents started by herdr" => {
            "Herdr 啟動或恢復的 Codex、Claude 將使用安全自動權限"
        }
        "sound alerts" => "音效提醒",
        "play sounds when agents change state in background" => "背景代理狀態改變時播放音效",
        "notification popups" => "彈出通知",
        "choose where background popup notifications should appear" => "選擇背景通知的顯示位置",
        "off" => "關閉",
        "on" => "開啟",
        "inside herdr" => "Herdr 內",
        "via terminal" => "終端機",
        "via system" => "系統通知",
        "agent border labels" => "代理邊框標籤",
        "show detected agent names in split pane borders" => "在分割窗格邊框顯示偵測到的代理名稱",
        "agent integrations" => "代理整合",
        "let agents report state directly instead of relying only on process detection" => {
            "讓代理直接回報狀態，不只依賴程序偵測"
        }
        "install" => "安裝",
        "apply" => "套用",
        "close" => "關閉",
        "select" => "選擇",
        "section" => "分類",
        "save" => "儲存",
        "clear" => "清除",
        "cancel" => "取消",
        "confirm" => "確認",
        "open" => "開啟",
        "remove" => "移除",
        "delete anyway" => "仍要刪除",
        "create and open" => "建立並開啟",
        "new workspace" => "新增工作空間",
        "rename workspace" => "重新命名工作空間",
        "new tab" => "新增分頁",
        "rename tab" => "重新命名分頁",
        "rename pane" => "重新命名窗格",
        "new worktree" => "新增工作樹",
        "branch" => "分支",
        "checkout" => "簽出目錄",
        "creating…" => "建立中…",
        "delete worktree checkout?" => "要刪除工作樹簽出目錄嗎？",
        "This removes the checkout folder:" => "這會移除以下簽出資料夾：",
        "The branch is not deleted. The Herdr workspace will close." => {
            "不會刪除分支，但 Herdr 工作空間會關閉。"
        }
        "Dirty or untracked files will be permanently deleted." => {
            "未提交或未追蹤的檔案將永久刪除。"
        }
        "removing…" => "移除中…",
        "open worktree" => "開啟工作樹",
        "filter worktrees" => "篩選工作樹",
        "no matching worktrees" => "沒有符合的工作樹",
        "checkouts" => "個簽出項目",
        "Close worktree group?" => "要關閉工作樹群組嗎？",
        "Close workspace?" => "要關閉工作空間嗎？",
        "spaces" => "工作空間",
        "agents" => "代理",
        "grouped" => "分組",
        "priority" => "優先",
        "filtered" => "已篩選",
        "new" => "新增",
        "menu" => "選單",
        "keybinds" => "快捷鍵",
        "reload config" => "重新載入設定",
        "update ready" => "可更新",
        "what's new" => "最新消息",
        "detach" => "分離",
        "Rename" => "重新命名",
        "Close" => "關閉",
        "New worktree" => "新增工作樹",
        "Open worktree..." => "開啟工作樹…",
        "Delete worktree checkout..." => "刪除工作樹簽出目錄…",
        "Close group" => "關閉群組",
        "Expand" => "展開",
        "Collapse" => "收合",
        "New tab" => "新增分頁",
        "Rename pane" => "重新命名窗格",
        "Clear pane name" => "清除窗格名稱",
        "Swap with focused pane" => "與目前窗格交換",
        "Split right" => "向右分割",
        "Split down" => "向下分割",
        "Zoom" => "縮放窗格",
        "Close pane" => "關閉窗格",
        "no matching agents" => "沒有符合的代理",
        "switch" => "切換",
        "no workspace" => "沒有工作空間",
        "no agents" => "沒有代理",
        "all idle" => "全部閒置",
        "blocked" => "待處理",
        "working" => "工作中",
        "done" => "已完成",
        "idle" => "閒置",
        "unknown" => "未知",
        "search panes" => "搜尋窗格",
        "shell" => "Shell",
        "continue" => "繼續",
        "terminal workspace manager for coding agents" => "為程式代理打造的終端工作空間管理器",
        "this is a mouse-first terminal." => "這是一個以滑鼠操作為主的終端介面。",
        "click the sidebar to switch workspaces, drag pane" => "點擊側欄切換工作空間，拖曳窗格",
        "borders to resize, right-click for context menus." => "邊框調整大小，按右鍵開啟快顯選單。",
        "enters prefix mode" => "進入前綴模式",
        "shows keybinds and settings" => "顯示快捷鍵與設定",
        "next: install optional agent integrations for more reliable state" => {
            "下一步：安裝選用的代理整合，讓狀態偵測更可靠"
        }
        _ => text,
    }
}

pub(crate) fn theme_name(language: UiLanguageConfig, name: &'static str) -> &'static str {
    if language == UiLanguageConfig::English {
        return name;
    }

    match name {
        "catppuccin" => "Catppuccin（深色）",
        "catppuccin-latte" => "Catppuccin Latte（淺色）",
        "terminal" => "跟隨終端機配色",
        "tokyo-night" => "Tokyo Night（深色）",
        "tokyo-night-day" => "Tokyo Night Day（淺色）",
        "dracula" => "Dracula（深色）",
        "nord" => "Nord（深色）",
        "gruvbox" => "Gruvbox（深色）",
        "gruvbox-light" => "Gruvbox（淺色）",
        "one-dark" => "One Dark（深色）",
        "one-light" => "One Light（淺色）",
        "solarized" => "Solarized（深色）",
        "solarized-light" => "Solarized（淺色）",
        "kanagawa" => "Kanagawa（深色）",
        "kanagawa-lotus" => "Kanagawa Lotus（淺色）",
        "rose-pine" => "Rosé Pine（深色）",
        "rose-pine-dawn" => "Rosé Pine Dawn（淺色）",
        "vesper" => "Vesper（深色）",
        _ => name,
    }
}

pub(crate) fn auto_mode_status(language: UiLanguageConfig, enabled: bool) -> String {
    match language {
        UiLanguageConfig::English => format!("auto: {}", if enabled { "on" } else { "off" }),
        UiLanguageConfig::TraditionalChinese => {
            format!("自動模式：{}", if enabled { "開" } else { "關" })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_traditional_chinese_and_preserves_english() {
        assert_eq!(tr(UiLanguageConfig::TraditionalChinese, "settings"), "設定");
        assert_eq!(tr(UiLanguageConfig::English, "settings"), "settings");
        assert_eq!(
            theme_name(UiLanguageConfig::TraditionalChinese, "terminal"),
            "跟隨終端機配色"
        );
        assert_eq!(
            auto_mode_status(UiLanguageConfig::TraditionalChinese, true),
            "自動模式：開"
        );
    }
}
