use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFile {
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

impl WorkspaceFile {
    pub fn label(&self) -> String {
        self.relative_path.display().to_string()
    }
}

pub fn collect_markdown_files(root: &Path) -> io::Result<Vec<WorkspaceFile>> {
    let mut files = Vec::new();
    visit_dir(root, root, &mut files)?;
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

fn visit_dir(root: &Path, dir: &Path, files: &mut Vec<WorkspaceFile>) -> io::Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            visit_dir(root, &path, files)?;
        } else if file_type.is_file() && is_markdown_path(&path) {
            let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            files.push(WorkspaceFile {
                path,
                relative_path,
            });
        }
    }

    Ok(())
}

fn is_markdown_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "markdown" | "mdown" | "mdx")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace_root(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("mdrs-workspace-{label}-{unique}"));
        path
    }

    #[test]
    fn collects_nested_markdown_files() {
        let root = temp_workspace_root("nested");
        let nested = root.join("docs");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("README.md"), "# hello").unwrap();
        fs::write(nested.join("guide.mdx"), "# guide").unwrap();
        fs::write(nested.join("notes.txt"), "ignore").unwrap();

        let files = collect_markdown_files(&root).unwrap();
        let labels = files.iter().map(WorkspaceFile::label).collect::<Vec<_>>();

        assert_eq!(labels, vec!["README.md", "docs\\guide.mdx"]);

        fs::remove_dir_all(root).unwrap();
    }
}
