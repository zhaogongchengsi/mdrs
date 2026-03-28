use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use gpui::{AssetSource, Result, SharedString};

pub struct AppAssets {
    base: PathBuf,
}

impl AppAssets {
    pub fn new() -> Self {
        Self {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets"),
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let mapped = match path {
            "icons/window-minimize.svg" => "icons/win-minimize.svg",
            "icons/window-maximize.svg" => "icons/win-maximize.svg",
            "icons/window-restore.svg" => "icons/win-restore.svg",
            "icons/window-close.svg" => "icons/win-close.svg",
            "icons/settings.svg" => "icons/setting.svg",
            _ => path,
        };

        self.base.join(mapped)
    }

    fn list_dir(path: &Path) -> Vec<SharedString> {
        fs::read_dir(path)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.flatten())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .map(SharedString::from)
            .collect()
    }
}

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let resolved = self.resolve_path(path);
        if !resolved.exists() {
            return Ok(None);
        }

        Ok(Some(Cow::Owned(fs::read(resolved)?)))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::list_dir(&self.base.join(path)))
    }
}
