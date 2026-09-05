use crate::{
    app::AppState,
    media::{MediaEntry, MediaKind},
};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::Paragraph,
    Frame,
};

pub(super) fn filtered_entries(app: &AppState) -> Vec<&MediaEntry> {
    let Some(ws) = app.active.and_then(|i| app.workspaces.get(i)) else {
        return Vec::new();
    };
    app.media_index
        .entries_for(&ws.id)
        .iter()
        .filter(|e| match app.media_ui.filter {
            1 => e.kind == MediaKind::Image,
            2 => e.kind == MediaKind::Document,
            _ => e.kind != MediaKind::Other,
        })
        .collect()
}
fn label(app: &AppState, zh: &'static str, en: &'static str) -> &'static str {
    if app.ui_language == crate::config::UiLanguageConfig::TraditionalChinese {
        zh
    } else {
        en
    }
}
fn line(frame: &mut Frame, area: Rect, row: u16, text: String, style: Style) {
    if row < area.height {
        frame.render_widget(
            Paragraph::new(text).style(style),
            Rect::new(area.x, area.y + row, area.width, 1),
        );
    }
}
pub(super) fn render_media_panel(app: &AppState, frame: &mut Frame, area: Rect) {
    if area.width < 2 || area.height == 0 {
        return;
    }
    let p = &app.palette;
    for y in area.y..area.bottom() {
        frame.buffer_mut()[(area.x, y)]
            .set_symbol("│")
            .set_style(Style::default().fg(p.surface_dim));
    }
    let inner = Rect::new(area.x + 1, area.y, area.width - 1, area.height);
    line(
        frame,
        inner,
        0,
        label(app, "專案圖片與文件", "Project media").into(),
        Style::default().fg(p.text).add_modifier(Modifier::BOLD),
    );
    let Some(ws) = app.active.and_then(|i| app.workspaces.get(i)) else {
        return;
    };
    let path = app
        .media_index
        .directory_for(&ws.id)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| ws.identity_cwd.join("media").display().to_string());
    line(
        frame,
        inner,
        1,
        super::text::truncate_end(&path, inner.width as usize),
        Style::default().fg(p.subtext0),
    );
    for (i, text) in [
        label(app, "開資料夾", "Folder"),
        label(app, "重新掃描", "Rescan"),
        label(app, "設定", "Settings"),
    ]
    .iter()
    .enumerate()
    {
        let x = area.x + area.width * i as u16 / 3;
        line(
            frame,
            Rect::new(
                x + 1,
                area.y,
                (area.width / 3).saturating_sub(1),
                area.height,
            ),
            2,
            text.to_string(),
            Style::default().fg(p.blue),
        );
    }
    let entries = filtered_entries(app);
    let filter = match app.media_ui.filter {
        1 => label(app, "圖片", "Images"),
        2 => label(app, "文件", "Documents"),
        _ => label(app, "全部", "All"),
    };
    line(
        frame,
        inner,
        3,
        format!("{filter} ▾  {}", entries.len()),
        Style::default().fg(p.accent),
    );
    if let Some(error) = &app.media_ui.error {
        line(frame, inner, 4, error.clone(), Style::default().fg(p.red));
    } else {
        line(
            frame,
            inner,
            4,
            label(
                app,
                "點擊預覽 · 右鍵開啟",
                "Click preview · right-click open",
            )
            .into(),
            Style::default().fg(p.overlay0),
        );
    }
    if entries.is_empty() {
        line(
            frame,
            inner,
            5,
            label(app, "拖入圖片或文件開始", "Drop an image or document").into(),
            Style::default().fg(p.overlay0),
        );
    }
    for (row, entry) in entries
        .iter()
        .skip(app.media_ui.scroll)
        .take(inner.height.saturating_sub(6) as usize)
        .enumerate()
    {
        let name = entry.path.file_name().unwrap_or_default().to_string_lossy();
        // Imports carry a full content hash for deduplication; show their readable name.
        let name = if name.len() > 65
            && name.as_bytes().get(64) == Some(&b'-')
            && name[..64].bytes().all(|b| b.is_ascii_hexdigit())
        {
            &name[65..]
        } else {
            &name
        };
        let glyph = if entry.kind == MediaKind::Image {
            "▧"
        } else {
            "▤"
        };
        let selected = app.media_ui.selected.as_ref() == Some(&entry.path);
        let style = Style::default()
            .fg(p.text)
            .bg(if selected { p.surface0 } else { p.panel_bg });
        line(
            frame,
            inner,
            5 + row as u16,
            format!(
                "{glyph} {}",
                super::text::truncate_end(name, inner.width.saturating_sub(2) as usize)
            ),
            style,
        );
    }
    line(
        frame,
        inner,
        inner.height.saturating_sub(1),
        format!("{} · m/M", label(app, "滾輪捲動", "Scroll")),
        Style::default().fg(p.overlay0),
    );
}

pub(super) fn render_media_dialog(app: &AppState, frame: &mut Frame, area: Rect) {
    let Some(dialog) = &app.media_ui.dialog else {
        return;
    };
    super::dim_background(frame, area);
    let Some(inner) = super::widgets::render_modal_shell(frame, area, 76, 17, &app.palette) else {
        return;
    };
    let editing = dialog.workspace_id.is_some();
    let title = if editing {
        label(app, "工作區媒體設定", "Workspace media settings")
    } else {
        label(app, "建立或開啟工作區", "Create or open workspace")
    };
    super::widgets::render_modal_header(frame, inner, title, &app.palette);
    let normal = Style::default().fg(app.palette.text);
    line(
        frame,
        inner,
        1,
        label(
            app,
            if editing {
                "專案位置"
            } else {
                "工作區名稱"
            },
            if editing {
                "Project location"
            } else {
                "Workspace name"
            },
        )
        .into(),
        normal,
    );
    line(
        frame,
        inner,
        2,
        format!(
            "{}{}",
            dialog.name,
            if dialog.field == 0 { "█" } else { "" }
        ),
        normal.bg(if dialog.field == 0 {
            app.palette.surface0
        } else {
            app.palette.panel_bg
        }),
    );
    line(
        frame,
        inner,
        3,
        if editing {
            label(app, "媒體資料夾完整路徑", "Media folder full path")
        } else {
            label(
                app,
                "專案資料夾完整路徑（自動建立 media/）",
                "Full project path (creates media/)",
            )
        }
        .into(),
        normal,
    );
    let input = super::text::truncate_end(&dialog.path, inner.width.saturating_sub(1) as usize);
    line(
        frame,
        inner,
        4,
        format!("{input}{}", if dialog.field == 1 { "█" } else { "" }),
        normal.bg(if dialog.field == 1 {
            app.palette.surface0
        } else {
            app.palette.panel_bg
        }),
    );
    let checkbox = if editing {
        label(
            app,
            "複製原媒體資料夾的檔案（保留原檔）",
            "Copy existing media (keep originals)",
        )
    } else {
        label(
            app,
            "開啟既有資料夾（不建立新專案）",
            "Open an existing folder",
        )
    };
    line(
        frame,
        inner,
        6,
        format!(
            "{} [{}] {checkbox}",
            if dialog.field == 2 { ">" } else { " " },
            if dialog.existing_or_copy { "x" } else { " " }
        ),
        normal,
    );
    if editing {
        line(
            frame,
            inner,
            7,
            format!(
                "{} [{}] {}",
                if dialog.field == 3 { ">" } else { " " },
                if dialog.collect_outputs { "x" } else { " " },
                label(
                    app,
                    "收集專案內新增或更新的圖片與文件",
                    "Collect new/updated project images and documents"
                )
            ),
            normal,
        );
    } else {
        line(
            frame,
            inner,
            7,
            label(
                app,
                "新專案會自動收集成品；既有專案可在設定開啟。",
                "New projects collect outputs; enable in settings for existing ones.",
            )
            .into(),
            normal,
        );
    }
    line(
        frame,
        inner,
        8,
        label(
            app,
            "代理共用此資料夾；更改路徑後請重新啟動代理。",
            "Agents share this folder; restart agents after changing its path.",
        )
        .into(),
        Style::default().fg(app.palette.subtext0),
    );
    if let Some(error) = &dialog.error {
        line(
            frame,
            inner,
            10,
            error.clone(),
            Style::default().fg(app.palette.red),
        );
    } else {
        line(
            frame,
            inner,
            10,
            label(
                app,
                "Tab 切換欄位 · 空白鍵勾選 · Ctrl+U 清除",
                "Tab fields · Space toggle · Ctrl+U clear",
            )
            .into(),
            Style::default().fg(app.palette.subtext0),
        );
    }
    line(
        frame,
        inner,
        12,
        label(app, "[ Enter 儲存 ]", "[ Enter Save ]").into(),
        normal.bg(app.palette.surface0),
    );
    line(
        frame,
        inner,
        13,
        label(app, "[ Esc 取消 ]", "[ Esc Cancel ]").into(),
        normal,
    );
}
