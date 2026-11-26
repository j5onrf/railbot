use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub content: String,
}

impl Skill {
    pub fn parse(name: &str, raw_content: &str) -> Self {
        let mut body = raw_content.to_string();
        let trimmed = raw_content.trim_start();
        if trimmed.starts_with("---") {
            let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
            if parts.len() >= 3 {
                body = parts[2].trim().to_string();
            }
        }
        Self { name: name.to_string(), content: body }
    }

    pub fn skill_dirs() -> Vec<PathBuf> {
        dirs::home_dir()
            .map(|h| vec![h.join(".config/railbot/skills")])
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.is_dir())
            .collect()
    }

    pub fn list_available() -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut list = Vec::new();
        for dir in Self::skill_dirs() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let lower = stem.to_lowercase();
                            if lower != "readme"
                                && lower != "system_instructions"
                                && lower != "instructions"
                                && seen.insert(stem.to_string())
                            {
                                list.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
        list.sort();
        list
    }

    pub fn load(skill_name: &str) -> Option<Self> {
        for dir in Self::skill_dirs() {
            let direct = dir.join(format!("{}.md", skill_name));
            if direct.is_file() {
                if let Ok(content) = fs::read_to_string(&direct) {
                    return Some(Self::parse(skill_name, &content));
                }
            }
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem.eq_ignore_ascii_case(skill_name) {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    return Some(Self::parse(stem, &content));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
