use crate::client::ChatMessage;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Default, Serialize, Deserialize)]
pub struct WorkspaceSession {
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui_history: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_stats: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

pub fn load_workspace_session(workspace: &Path) -> WorkspaceSession {
    fs::read_to_string(workspace.join(".rs/session.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_workspace_session(workspace: &Path, session: &WorkspaceSession) {
    let dir = workspace.join(".rs");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let Ok(content) = serde_json::to_string(session) else {
        return;
    };
    static TMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let final_path = dir.join("session.json");
    let tmp_path = dir.join(format!("session.json.tmp.{}.{}", std::process::id(), seq));
    if fs::write(&tmp_path, content).is_err() {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
    }
    let _ = fs::rename(&tmp_path, &final_path);
}

pub fn clear_workspace_session(workspace: &Path) {
    let _ = fs::remove_file(workspace.join(".rs/session.json"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_session_roundtrip_and_clear() {
        let tmp = std::env::temp_dir().join(format!("railbot_sess_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        assert!(load_workspace_session(&tmp).messages.is_empty());

        let snap = WorkspaceSession {
            messages: vec![ChatMessage::user("hello"), ChatMessage::assistant("hi")],
            active_skills: vec!["ponytail".into()],
            reasoning: Some("250".into()),
            ..Default::default()
        };
        save_workspace_session(&tmp, &snap);
        let loaded = load_workspace_session(&tmp);
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].content_as_str(), "hello");
        assert_eq!(loaded.active_skills, vec!["ponytail"]);
        assert_eq!(loaded.reasoning.as_deref(), Some("250"));

        clear_workspace_session(&tmp);
        assert!(load_workspace_session(&tmp).messages.is_empty());
        let _ = fs::remove_dir_all(&tmp);
    }
}
