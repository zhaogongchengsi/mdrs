use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

const STYLE_DIR: &str = ".mdrs";
const STYLE_FILE: &str = "theme.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MarkdownStyle {
    pub heading_h1_size: f32,
    pub heading_h2_size: f32,
    pub heading_h3_size: f32,
    pub heading_h4_size: f32,
    pub heading_h5_size: f32,
    pub heading_h6_size: f32,
    pub paragraph_size: f32,
    pub inline_code_size: f32,
    pub code_block_size: f32,
    pub block_gap: f32,
}

impl Default for MarkdownStyle {
    fn default() -> Self {
        Self {
            heading_h1_size: 32.0,
            heading_h2_size: 26.0,
            heading_h3_size: 22.0,
            heading_h4_size: 18.0,
            heading_h5_size: 16.0,
            heading_h6_size: 14.0,
            paragraph_size: 15.0,
            inline_code_size: 13.0,
            code_block_size: 13.0,
            block_gap: 12.0,
        }
    }
}

impl MarkdownStyle {
    pub fn config_path() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(STYLE_DIR).join(STYLE_FILE)
    }
}

pub fn load_markdown_style() -> MarkdownStyle {
    let path = MarkdownStyle::config_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(parsed) = toml::from_str::<MarkdownStyle>(&raw) {
            return parsed;
        }
    }

    let style = MarkdownStyle::default();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(serialized) = toml::to_string_pretty(&style) {
        let _ = fs::write(path, serialized);
    }

    style
}
