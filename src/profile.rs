use crate::config::{strip_inline_comment, ToolTier};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasoningSetting {
    Tokens(usize),
    Tier(String),
}

impl ReasoningSetting {
    pub fn parse(raw: &str) -> Option<Self> {
        let clean = raw.trim().trim_matches('"').trim_matches('\'').trim();
        if let Ok(n) = clean.parse::<usize>() {
            return Some(Self::Tokens(n));
        }
        let lower = clean.to_ascii_lowercase();
        if matches!(lower.as_str(), "none" | "minimal" | "low" | "medium" | "high" | "xhigh" | "max") {
            return Some(Self::Tier(lower));
        }
        None
    }

    pub fn to_local_tokens(&self) -> usize {
        match self {
            Self::Tokens(n) => *n,
            Self::Tier(t) => match t.as_str() {
                "none" | "minimal" => 0,
                "low" => 250,
                "medium" => 500,
                "high" => 1000,
                "xhigh" | "max" => 4000,
                _ => 500,
            },
        }
    }

    pub fn to_cloud_tier(&self, is_google: bool, model: &str) -> String {
        let raw = match self {
            Self::Tier(t) => t.as_str(),
            Self::Tokens(n) => {
                if *n == 0 {
                    "none"
                } else if *n <= 350 {
                    "low"
                } else if *n <= 750 {
                    "medium"
                } else if *n <= 2000 {
                    "high"
                } else {
                    "xhigh"
                }
            }
        };

        if is_google {
            let m = model.to_lowercase();
            let no_minimal =
                m.contains("gemini-4") || m.contains("gemini-3.8") || m.contains("gemini-3.7") || m.contains("-pro");
            let no_none = m.contains("gemini-3") || m.contains("gemini-4") || m.contains("pro");
            if no_minimal && matches!(raw, "none" | "minimal") {
                return "low".to_string();
            }
            if no_none && raw == "none" {
                return "minimal".to_string();
            }
            if matches!(raw, "xhigh" | "max") {
                return if m.contains("gemini-4-hydrogen") { "xhigh" } else { "high" }.to_string();
            }
        }
        raw.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub reasoning_budget: Option<ReasoningSetting>,
    pub tier: ToolTier,
    pub adapters: bool,
    pub memory: bool,
    pub delegate: bool,
    pub system_prompt: String,
    pub mcp_servers: Vec<String>,
    pub mcp_tools: Vec<String>,
    pub ctx: Option<usize>,
}

impl Profile {
    pub fn parse(raw_content: &str) -> Self {
        let mut reasoning_budget = None;
        let mut tier = ToolTier::Smol;
        let mut adapters = false;
        let mut memory = false;
        let mut delegate = false;
        let mut mcp_servers = Vec::new();
        let mut mcp_tools = Vec::new();
        let mut ctx = None;
        let mut body = String::new();
        let mut in_frontmatter = false;
        let mut frontmatter_closed = false;

        for line in raw_content.lines() {
            let trimmed = line.trim();
            if trimmed == "---" {
                if !in_frontmatter && !frontmatter_closed {
                    in_frontmatter = true;
                    continue;
                } else if in_frontmatter {
                    in_frontmatter = false;
                    frontmatter_closed = true;
                    continue;
                }
            }

            if in_frontmatter {
                if let Some((k, v)) = trimmed.split_once(':') {
                    let key = k.trim();
                    let val = strip_inline_comment(v.trim());
                    match key {
                        "reasoning_budget" => {
                            if let Some(rs) = ReasoningSetting::parse(val) {
                                reasoning_budget = Some(rs);
                            }
                        }
                        "tier" if val.eq_ignore_ascii_case("purebash") => {
                            tier = ToolTier::PureBash;
                        }
                        "tier" if val.eq_ignore_ascii_case("core4") => {
                            tier = ToolTier::Core4;
                        }
                        "purebash" if val.eq_ignore_ascii_case("true") => {
                            tier = ToolTier::PureBash;
                        }
                        "core4" if val.eq_ignore_ascii_case("true") => {
                            tier = ToolTier::Core4;
                        }
                        "adapters" => {
                            adapters = val.eq_ignore_ascii_case("true");
                        }
                        "memory" => {
                            memory = val.eq_ignore_ascii_case("true");
                        }
                        "delegate" | "subagent" | "subagents" => {
                            delegate = val.eq_ignore_ascii_case("true");
                        }
                        "mcp" => {
                            let clean = val.trim_matches('[').trim_matches(']').trim();
                            for item in clean.split(',') {
                                let s = item.trim().trim_matches('"').trim_matches('\'').trim();
                                if !s.is_empty() {
                                    mcp_servers.push(s.to_string());
                                }
                            }
                        }
                        "ctx" | "context_size" => {
                            let s = val.trim_matches(['"', '\'']).trim();
                            let n = s.parse::<usize>().ok()
                                .or_else(|| s.strip_suffix(['k', 'K']).and_then(|v| v.parse::<usize>().ok().map(|x| x.saturating_mul(1024))))
                                .or_else(|| s.strip_suffix(['m', 'M']).and_then(|v| v.parse::<usize>().ok().map(|x| x.saturating_mul(1048576))));
                            if let Some(x) = n.filter(|&x| x > 0) {
                                ctx = Some(x);
                            }
                        }
                        "mcp_tools" => {
                            let clean = val.trim_matches('[').trim_matches(']').trim();
                            for item in clean.split(',') {
                                let s = item.trim().trim_matches('"').trim_matches('\'').trim();
                                if !s.is_empty() {
                                    mcp_tools.push(s.to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }

        Self {
            reasoning_budget,
            tier,
            adapters,
            memory,
            delegate,
            system_prompt: body.trim().to_string(),
            mcp_servers,
            mcp_tools,
            ctx,
        }
    }

    pub fn hud_summary(&self, is_agent: bool) -> String {
        let mut parts: Vec<String> = Vec::new();
        if is_agent {
            match self.tier {
                ToolTier::PureBash => parts.push("purebash".to_string()),
                ToolTier::Core4 => parts.push("core4".to_string()),
                ToolTier::Smol => {}
            }
        } else {
            parts.push("no tools".to_string());
        }
        if let Some(ref rb) = self.reasoning_budget {
            let v = match rb {
                ReasoningSetting::Tokens(n) => n.to_string(),
                ReasoningSetting::Tier(t) => t.clone(),
            };
            parts.push(v);
        }
        if self.adapters {
            parts.push("adp".to_string());
        }
        if self.memory {
            parts.push("mem".to_string());
        }
        if self.delegate {
            parts.push("del".to_string());
        }
        if !self.mcp_servers.is_empty() {
            if self.mcp_tools.is_empty() {
                parts.push(format!("mcp:{}", self.mcp_servers.join(",")));
            } else {
                parts.push(format!("mcp:{}({})", self.mcp_servers.join(","), self.mcp_tools.len()));
            }
        }
        if !is_agent && parts.len() == 1 {
            parts.push("pure chat".to_string());
        }
        parts.join(" · ")
    }

    pub fn profile_dirs() -> Vec<PathBuf> {
        dirs::home_dir()
            .map(|h| vec![h.join(".config/railbot/profiles")])
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.is_dir())
            .collect()
    }

    pub fn load(profile_name: &str) -> Option<Self> {
        let norm_target = profile_name.replace(['-', '_'], "").to_lowercase();
        let target_stem = profile_name.split('/').next_back().unwrap_or(profile_name);
        let target_stem_norm = target_stem.replace(['-', '_'], "").to_lowercase();

        for dir in Self::profile_dirs() {
            let direct = dir.join(format!("{}.md", profile_name));
            if direct.is_file() {
                if let Ok(content) = fs::read_to_string(&direct) {
                    return Some(Self::parse(&content));
                }
            }

            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        if dir_name.starts_with('.') {
                            continue;
                        }

                        let sub_direct = path.join(format!("{}.md", target_stem));
                        if sub_direct.is_file() {
                            if let Some((cat, _)) = profile_name.split_once('/') {
                                if cat.eq_ignore_ascii_case(&dir_name) {
                                    if let Ok(content) = fs::read_to_string(&sub_direct) {
                                        return Some(Self::parse(&content));
                                    }
                                }
                            } else if let Ok(content) = fs::read_to_string(&sub_direct) {
                                return Some(Self::parse(&content));
                            }
                        }

                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if sub_path.extension().and_then(|e| e.to_str()) == Some("md") {
                                    if let Some(stem) = sub_path.file_stem().and_then(|s| s.to_str()) {
                                        let full_id = format!("{}/{}", dir_name, stem);
                                        let full_norm = full_id.replace(['-', '_'], "").to_lowercase();
                                        let stem_norm = stem.replace(['-', '_'], "").to_lowercase();

                                        if full_norm == norm_target
                                            || stem_norm == norm_target
                                            || stem_norm == target_stem_norm
                                        {
                                            if let Ok(content) = fs::read_to_string(&sub_path) {
                                                return Some(Self::parse(&content));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem.replace(['-', '_'], "").to_lowercase() == norm_target {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    return Some(Self::parse(&content));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn list_available() -> Vec<String> {
        let mut seen = HashSet::new();
        let mut list = Vec::new();

        for dir in Self::profile_dirs() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        if dir_name.starts_with('.') {
                            continue;
                        }
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if sub_path.extension().and_then(|e| e.to_str()) == Some("md") {
                                    if let Some(stem) = sub_path.file_stem().and_then(|s| s.to_str()) {
                                        let id = format!("{}/{}", dir_name, stem);
                                        if stem != "chat" && seen.insert(id.clone()) {
                                            list.push(id);
                                        }
                                    }
                                }
                            }
                        }
                    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if stem != "chat" && seen.insert(stem.to_string()) {
                                list.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }

        list.sort_by(|a, b| {
            let a_nested = a.contains('/');
            let b_nested = b.contains('/');
            match (a_nested, b_nested) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.cmp(b),
            }
        });

        list
    }

    pub fn find_workspace_candidates(ws_name: &str, available: &[String]) -> Vec<String> {
        let ws_clean = ws_name.replace(['-', '_'], "").to_lowercase();
        let mut exact = Vec::new();
        let mut prefix = Vec::new();

        for opt in available {
            let bare = opt.split('/').next_back().unwrap_or(opt);
            let opt_clean = opt.replace(['-', '_'], "").to_lowercase();
            let bare_clean = bare.replace(['-', '_'], "").to_lowercase();

            if opt_clean == ws_clean || bare_clean == ws_clean {
                exact.push(opt.clone());
            } else if ws_clean.len() >= 2
                && (opt_clean.starts_with(&ws_clean) || bare_clean.starts_with(&ws_clean))
                && !prefix.contains(opt)
            {
                prefix.push(opt.clone());
            }
        }

        if !exact.is_empty() {
            exact
        } else {
            prefix
        }
    }

}

pub fn build_system_prompt(profile: &Profile, workspace: &Path, is_agent: bool, active_skills: &[String]) -> String {
    let mut prompt = if is_agent {
        format!("{}\n\nWorkspace Path: {}", profile.system_prompt, workspace.display())
    } else {
        profile.system_prompt.clone()
    };

    if let Some(global_rules) = crate::config::load_global_instructions() {
        prompt.push_str(&format!("\n\n---\nGLOBAL SYSTEM INSTRUCTIONS:\n{}", global_rules));
    }

    if is_agent && profile.memory {
        let mem_path = workspace.join(".rs/memory.md");
        if let Ok(c) = fs::read_to_string(mem_path) {
            let trimmed = c.trim();
            if !trimmed.is_empty() {
                prompt.push_str(&format!("\n\n---\nWORKSPACE MEMORY (.rs/memory.md):\n{}", trimmed));
            }
        }
    }

    for sk_name in active_skills {
        if let Some(sk) = crate::skill::Skill::load(sk_name) {
            prompt.push_str(&format!("\n\n---\nSKILL DIRECTIVE [{}]:\n{}", sk.name, sk.content));
        }
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_parses_cloud_effort_and_core4() {
        let raw = "---\ncore4: true # minimal tools\nreasoning_budget: xhigh # cloud tier\nadapters: true\n---\nSystem prompt";
        let prof = Profile::parse(raw);
        assert!(matches!(prof.tier, ToolTier::Core4));
        assert_eq!(prof.reasoning_budget, Some(ReasoningSetting::Tier("xhigh".into())));
        assert!(prof.adapters);
    }

    #[test]
    fn test_google_cloud_tier_clamping() {
        assert_eq!(ReasoningSetting::Tokens(0).to_cloud_tier(true, "gemini-3.5-flash-lite"), "minimal");
        assert_eq!(ReasoningSetting::Tier("xhigh".into()).to_cloud_tier(true, "gemini-3.5-flash-lite"), "high");
        assert_eq!(ReasoningSetting::Tier("xhigh".into()).to_cloud_tier(true, "gemini-4-hydrogen"), "xhigh");
        assert_eq!(ReasoningSetting::Tier("none".into()).to_cloud_tier(true, "gemini-4-hydrogen"), "low");
    }

    #[test]
    fn test_reasoning_budget_parses_tokens_and_tiers() {
        let local = Profile::parse("---\nreasoning_budget: 500\n---\nhi");
        assert_eq!(local.reasoning_budget, Some(ReasoningSetting::Tokens(500)));
        let cloud = Profile::parse("---\nreasoning_budget: high\n---\nhi");
        assert_eq!(cloud.reasoning_budget, Some(ReasoningSetting::Tier("high".into())));
    }

    #[test]
    fn test_hud_summary_formats_frontmatter() {
        let ling = Profile::parse("---\nadapters: true\nreasoning_budget: 350\n---\nhi");
        assert_eq!(ling.hud_summary(true), "350 · adp");

        let gem = Profile::parse("---\ncore4: true\nreasoning_budget: xhigh\nmemory: true\ndelegate: true\nmcp: photocraft\n---\nhi");
        assert_eq!(gem.hud_summary(true), "core4 · xhigh · mem · del · mcp:photocraft");

        let chat = Profile::parse("hi");
        assert_eq!(chat.hud_summary(false), "no tools · pure chat");
    }

    #[test]
    fn test_find_workspace_candidates_resolves_ling_and_exact_groot() {
        let avail = vec![
            "artcraft/photocraft-c".to_string(),
            "artcraft/photocraft-l".to_string(),
            "groot/groot".to_string(),
            "groot/groot2".to_string(),
            "lfm3".to_string(),
            "lingtiny".to_string(),
        ];
        assert_eq!(Profile::find_workspace_candidates("lin", &avail), vec!["lingtiny"]);
        assert_eq!(Profile::find_workspace_candidates("ling", &avail), vec!["lingtiny"]);
        assert_eq!(Profile::find_workspace_candidates("ling-tiny", &avail), vec!["lingtiny"]);
        assert_eq!(Profile::find_workspace_candidates("lfm", &avail), vec!["lfm3"]);
        assert_eq!(Profile::find_workspace_candidates("groot", &avail), vec!["groot/groot"]);
        assert_eq!(
            Profile::find_workspace_candidates("artcraft", &avail),
            vec!["artcraft/photocraft-c", "artcraft/photocraft-l"]
        );
    }

    #[test]
    fn test_profile_parses_mcp_tools_allowlist() {
        let p = Profile::parse("---\nmcp: photocraft\nmcp_tools: doc_new, doc_save, ui_set\n---\nhi");
        assert_eq!(p.mcp_servers, vec!["photocraft"]);
        assert_eq!(p.mcp_tools, vec!["doc_new", "doc_save", "ui_set"]);
        assert_eq!(p.hud_summary(true), "mcp:photocraft(3)");
    }

    #[test]
    fn test_profile_parses_ctx_override() {
        assert_eq!(Profile::parse("---\nctx: 65536\n---\nhi").ctx, Some(65536));
        assert_eq!(Profile::parse("---\nctx: 64k\n---\nhi").ctx, Some(65536));
        assert_eq!(Profile::parse("---\nctx: 128k\n---\nhi").ctx, Some(131072));
        assert_eq!(Profile::parse("---\nctx: 1m\n---\nhi").ctx, Some(1048576));
    }

}
