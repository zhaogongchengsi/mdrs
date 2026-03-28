use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};

const DIRECT_READ_LIMIT: u64 = 1024 * 1024;
const BUFFER_CAPACITY: usize = 256 * 1024;
const MAX_PREALLOC_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadStrategy {
    Direct,
    Buffered,
}

impl ReadStrategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct read",
            Self::Buffered => "buffered streaming read",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedMarkdown {
    pub path: PathBuf,
    pub text: String,
    pub bytes: u64,
    pub read_strategy: ReadStrategy,
}

#[derive(Debug)]
pub enum LoadMarkdownError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    InvalidUtf8 {
        path: PathBuf,
        source: std::string::FromUtf8Error,
    },
    TooLarge {
        path: PathBuf,
        bytes: u64,
    },
}

impl LoadMarkdownError {
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. } => path,
            Self::InvalidUtf8 { path, .. } => path,
            Self::TooLarge { path, .. } => path,
        }
    }
}

impl fmt::Display for LoadMarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { source, .. } => write!(f, "{source}"),
            Self::InvalidUtf8 { source, .. } => write!(f, "file is not valid UTF-8: {source}"),
            Self::TooLarge { bytes, .. } => write!(
                f,
                "file is too large to load into the editor ({})",
                format_bytes(*bytes)
            ),
        }
    }
}

impl std::error::Error for LoadMarkdownError {}

pub fn load_markdown_file(path: impl Into<PathBuf>) -> Result<LoadedMarkdown, LoadMarkdownError> {
    let path = path.into();
    let metadata = fs::metadata(&path).map_err(|source| LoadMarkdownError::Io {
        path: path.clone(),
        source,
    })?;
    let bytes = metadata.len();

    if bytes > usize::MAX as u64 {
        return Err(LoadMarkdownError::TooLarge { path, bytes });
    }

    let (raw, read_strategy) = if bytes <= DIRECT_READ_LIMIT {
        let raw = fs::read(&path).map_err(|source| LoadMarkdownError::Io {
            path: path.clone(),
            source,
        })?;
        (raw, ReadStrategy::Direct)
    } else {
        let file = File::open(&path).map_err(|source| LoadMarkdownError::Io {
            path: path.clone(),
            source,
        })?;
        let mut reader = BufReader::with_capacity(BUFFER_CAPACITY, file);
        let capacity = usize::try_from(bytes)
            .unwrap_or(MAX_PREALLOC_BYTES)
            .min(MAX_PREALLOC_BYTES);
        let mut raw = Vec::with_capacity(capacity);
        reader
            .read_to_end(&mut raw)
            .map_err(|source| LoadMarkdownError::Io {
                path: path.clone(),
                source,
            })?;
        (raw, ReadStrategy::Buffered)
    };

    let text = String::from_utf8(raw).map_err(|source| LoadMarkdownError::InvalidUtf8 {
        path: path.clone(),
        source,
    })?;

    Ok(LoadedMarkdown {
        path,
        text,
        bytes,
        read_strategy,
    })
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_markdown_path(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("mdrs-{label}-{unique}.md"));
        path
    }

    #[test]
    fn loads_small_files_with_direct_read() {
        let path = temp_markdown_path("small");
        fs::write(&path, "# small\nhello").unwrap();

        let loaded = load_markdown_file(path.clone()).unwrap();
        assert_eq!(loaded.read_strategy, ReadStrategy::Direct);
        assert_eq!(loaded.text, "# small\nhello");

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn loads_large_files_with_buffered_read() {
        let path = temp_markdown_path("large");
        let large_markdown = format!(
            "# large\n{}",
            "content line\n".repeat((DIRECT_READ_LIMIT as usize / 12) + 2048)
        );
        fs::write(&path, &large_markdown).unwrap();

        let loaded = load_markdown_file(path.clone()).unwrap();
        assert_eq!(loaded.read_strategy, ReadStrategy::Buffered);
        assert_eq!(loaded.text, large_markdown);

        fs::remove_file(path).unwrap();
    }
}
