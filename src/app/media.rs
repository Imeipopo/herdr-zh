//! Media presentation/controller glue. Filesystem policy stays in crate::media.
use super::{App, Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct MediaUi {
    pub dialog: Option<MediaDialog>,
    pub area: Rect,
    pub scroll: usize,
    pub filter: u8,
    pub selected: Option<PathBuf>,
    pub workspace: Option<String>,
    pub error: Option<String>,
}
#[derive(Debug, Clone)]
pub struct MediaDialog {
    pub workspace_id: Option<String>,
    pub name: String,
    pub path: String,
    pub existing_or_copy: bool,
    pub collect_outputs: bool,
    pub field: u8,
    pub replace: bool,
    pub error: Option<String>,
}
impl MediaDialog {
    pub fn new_project(path: PathBuf) -> Self {
        Self {
            workspace_id: None,
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            path: path.display().to_string(),
            existing_or_copy: false,
            collect_outputs: true,
            field: 0,
            replace: true,
            error: None,
        }
    }
}

pub fn dialog_inner(area: Rect) -> Option<Rect> {
    // Same geometry as the existing modal shell.
    let width = 76.min(area.width.saturating_sub(4));
    let height = 17.min(area.height.saturating_sub(2));
    if width < 4 || height < 4 {
        return None;
    }
    Some(Rect::new(
        area.x + (area.width - width) / 2 + 1,
        area.y + (area.height - height) / 2 + 1,
        width - 2,
        height - 2,
    ))
}

impl App {
    pub(crate) fn media_error(&mut self, message: String) {
        tracing::warn!(%message, "media operation failed");
        self.state.media_ui.error = Some(message);
        self.state.media_panel_collapsed = false;
    }

    pub(crate) fn open_media_settings(&mut self) {
        let Some(index) = self.state.active else {
            return;
        };
        let Some(ws) = self.state.workspaces.get(index) else {
            return;
        };
        let settings = crate::media::MediaSettings::load(&ws.identity_cwd);
        self.state.media_ui.dialog = Some(MediaDialog {
            workspace_id: Some(self.public_workspace_id(index)),
            name: ws.identity_cwd.display().to_string(),
            path: settings.directory.display().to_string(),
            existing_or_copy: true,
            collect_outputs: settings.collect_outputs,
            field: 1,
            replace: true,
            error: None,
        });
        self.state.mode = Mode::RenameWorkspace;
    }

    pub(crate) fn insert_media_text(&mut self, text: &str) {
        let Some(dialog) = &mut self.state.media_ui.dialog else {
            return;
        };
        if dialog.field > 1 {
            return;
        }
        let old_name = dialog.name.clone();
        let field = if dialog.field == 0 {
            &mut dialog.name
        } else {
            &mut dialog.path
        };
        if dialog.replace {
            field.clear();
            dialog.replace = false;
        }
        field.extend(text.chars().filter(|c| !c.is_control()));
        if dialog.field == 0 && !dialog.existing_or_copy && dialog.workspace_id.is_none() {
            let path = PathBuf::from(&dialog.path);
            if path
                .file_name()
                .is_some_and(|name| name == old_name.as_str())
            {
                let sanitized = crate::media::project_dir_for_name(&dialog.name);
                dialog.path = path
                    .with_file_name(sanitized.file_name().unwrap_or_default())
                    .display()
                    .to_string();
            }
        }
        dialog.error = None;
        self.state.name_input = dialog.name.clone();
    }

    pub(crate) fn handle_media_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.state.media_ui.dialog = None;
            self.state.pending_workspace_create_cwd = None;
            self.state.mode = if self.state.active.is_some() {
                Mode::Terminal
            } else {
                Mode::Navigate
            };
            return;
        }
        if key.code == KeyCode::Enter {
            self.save_media_dialog();
            return;
        }
        let Some(dialog) = &mut self.state.media_ui.dialog else {
            return;
        };
        let first = u8::from(dialog.workspace_id.is_some());
        let last = if dialog.workspace_id.is_some() { 3 } else { 2 };
        match key.code {
            KeyCode::Tab | KeyCode::Down => {
                dialog.field = if dialog.field >= last {
                    first
                } else {
                    dialog.field + 1
                };
                dialog.replace = true;
            }
            KeyCode::BackTab | KeyCode::Up => {
                dialog.field = if dialog.field <= first {
                    last
                } else {
                    dialog.field - 1
                };
                dialog.replace = true;
            }
            KeyCode::Char(' ') if dialog.field == 2 => {
                dialog.existing_or_copy = !dialog.existing_or_copy
            }
            KeyCode::Char(' ') if dialog.field == 3 => {
                dialog.collect_outputs = !dialog.collect_outputs
            }
            KeyCode::Char('u' | 'c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if dialog.field == 0 {
                    dialog.name.clear();
                } else if dialog.field == 1 {
                    dialog.path.clear();
                }
                dialog.replace = false;
            }
            KeyCode::Backspace => {
                let field = if dialog.field == 0 {
                    &mut dialog.name
                } else if dialog.field == 1 {
                    &mut dialog.path
                } else {
                    return;
                };
                if dialog.replace {
                    field.clear();
                } else {
                    field.pop();
                }
                dialog.replace = false;
            }
            KeyCode::Char(c)
                if !key.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                ) =>
            {
                self.insert_media_text(&c.to_string())
            }
            _ => {}
        }
    }

    fn save_media_dialog(&mut self) {
        let Some(dialog) = self.state.media_ui.dialog.clone() else {
            return;
        };
        use crate::api::schema::{Method, ProjectCreateParams, WorkspaceMediaSetParams};
        let method = if let Some(workspace_id) = dialog.workspace_id {
            Method::WorkspaceMediaSet(WorkspaceMediaSetParams {
                workspace_id,
                directory: dialog.path,
                copy_existing: dialog.existing_or_copy,
                collect_outputs: dialog.collect_outputs,
            })
        } else {
            Method::ProjectCreate(ProjectCreateParams {
                path: dialog.path,
                label: dialog.name,
                open_existing: dialog.existing_or_copy,
            })
        };
        let response = self.dispatch_runtime_mutation("tui.media.configure", method);
        let result = serde_json::from_str::<serde_json::Value>(&response);
        match result {
            Ok(value) if value.get("error").is_none() => {
                self.state.media_ui.dialog = None;
                self.state.media_ui.error = None;
                self.state.media_ui.scroll = 0;
                self.state.pending_workspace_create_cwd = None;
                self.state.media_panel_collapsed = false;
                self.state.mode = Mode::Terminal;
                self.ensure_media_watchers();
            }
            value => {
                let message = value
                    .ok()
                    .and_then(|v| v["error"]["message"].as_str().map(str::to_owned))
                    .unwrap_or(response);
                if let Some(dialog) = &mut self.state.media_ui.dialog {
                    dialog.error = Some(message);
                }
            }
        }
    }

    pub(crate) fn handle_media_mouse(&mut self, mouse: MouseEvent) -> bool {
        if self.state.media_ui.dialog.is_some() {
            if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                let area = self.state.media_ui.area;
                if let Some(inner) = dialog_inner(area) {
                    if inner.contains((mouse.column, mouse.row).into()) {
                        let row = mouse.row - inner.y;
                        if row == 12 {
                            self.save_media_dialog();
                        } else if row == 13 {
                            self.handle_media_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
                        } else if let Some(dialog) = &mut self.state.media_ui.dialog {
                            match row {
                                2 if dialog.workspace_id.is_none() => {
                                    dialog.field = 0;
                                    dialog.replace = true;
                                }
                                4 => {
                                    dialog.field = 1;
                                    dialog.replace = true;
                                }
                                6 => {
                                    dialog.field = 2;
                                    dialog.existing_or_copy = !dialog.existing_or_copy;
                                }
                                7 if dialog.workspace_id.is_some() => {
                                    dialog.field = 3;
                                    dialog.collect_outputs = !dialog.collect_outputs;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            return true;
        }
        if !matches!(
            self.state.mode,
            Mode::Terminal | Mode::Navigate | Mode::Prefix
        ) {
            return false;
        }
        if self.state.toast.is_some()
            && self
                .state
                .view
                .toast_hit_area
                .contains((mouse.column, mouse.row).into())
        {
            return false;
        }
        let rect = self.state.view.media_panel_rect;
        if !rect.contains((mouse.column, mouse.row).into()) {
            return false;
        }
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.state.media_ui.scroll = self.state.media_ui.scroll.saturating_add(3)
            }
            MouseEventKind::ScrollUp => {
                self.state.media_ui.scroll = self.state.media_ui.scroll.saturating_sub(3)
            }
            MouseEventKind::Down(button) => {
                let row = mouse.row - rect.y;
                if row == 2 {
                    let part = (mouse.column - rect.x).saturating_mul(3) / rect.width.max(1);
                    match part {
                        0 => {
                            if let Some(ws) =
                                self.state.active.and_then(|i| self.state.workspaces.get(i))
                            {
                                let path = crate::media::directory(&ws.identity_cwd);
                                if let Err(err) =
                                    crate::platform::open_url(&path.display().to_string())
                                {
                                    self.media_error(err.to_string());
                                }
                            }
                        }
                        1 => {
                            if let Some(index) = self.state.active {
                                self.dispatch_runtime_mutation(
                                    "tui.media.rescan",
                                    crate::api::schema::Method::WorkspaceMediaGet(
                                        crate::api::schema::WorkspaceTarget {
                                            workspace_id: self.public_workspace_id(index),
                                        },
                                    ),
                                );
                            }
                            self.state.media_ui.error = None;
                        }
                        _ => self.open_media_settings(),
                    }
                } else if row == 3 {
                    self.state.media_ui.filter = (self.state.media_ui.filter + 1) % 3;
                    self.state.media_ui.scroll = 0;
                } else if let Some(path) = self
                    .state
                    .view
                    .media_entry_hit_areas
                    .iter()
                    .find(|hit| hit.rect.contains((mouse.column, mouse.row).into()))
                    .map(|hit| hit.path.clone())
                {
                    self.state.media_ui.selected = Some(path.clone());
                    let result = if button == MouseButton::Right {
                        crate::platform::open_url(&path.display().to_string())
                    } else {
                        crate::platform::preview_path(&path)
                    };
                    if let Err(err) = result {
                        self.media_error(err.to_string());
                    }
                }
            }
            _ => {}
        }
        true
    }
}
