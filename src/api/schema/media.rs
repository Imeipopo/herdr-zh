use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProjectCreateParams {
    /// Full project directory. Must not exist unless open_existing is true.
    pub path: String,
    pub label: String,
    #[serde(default)]
    pub open_existing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WorkspaceMediaSetParams {
    pub workspace_id: String,
    pub directory: String,
    #[serde(default)]
    pub collect_outputs: bool,
    #[serde(default)]
    pub copy_existing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MediaFileInfo {
    pub path: String,
    pub kind: String,
    pub modified_unix_seconds: u64,
}
