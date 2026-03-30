use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

// Embed only the SVG icons the UI needs at runtime.
#[derive(RustEmbed)]
#[folder = "src/assets"]
#[include = "icons/**/*.svg"]
pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Ok(Self::get(path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|asset_path| {
                asset_path
                    .starts_with(path)
                    .then(|| asset_path.as_ref().to_owned().into())
            })
            .collect())
    }
}
