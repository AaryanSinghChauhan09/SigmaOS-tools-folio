#![forbid(unsafe_code)]
//! folio - File manager for SigmaOS

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: EntryKind,
    pub size_bytes: u64,
    pub extension: Option<String>,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let extension = path.extension().map(|e| e.to_string_lossy().to_string());
        let kind = if path.is_dir() {
            EntryKind::Directory
        } else if path.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::File
        };
        let size_bytes = if kind == EntryKind::File {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        Self {
            name,
            path: path.to_path_buf(),
            kind,
            size_bytes,
            extension,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.name.starts_with('.')
    }

    pub fn is_dir(&self) -> bool {
        self.kind == EntryKind::Directory
    }

    pub fn human_size(&self) -> String {
        match self.size_bytes {
            0 => "0 B".to_string(),
            b if b < 1024 => format!("{} B", b),
            b if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
            b if b < 1024 * 1024 * 1024 => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
            b => format!("{:.1} GB", b as f64 / (1024.0 * 1024.0 * 1024.0)),
        }
    }
}

pub struct FileManager {
    pub current_dir: PathBuf,
    history: Vec<PathBuf>,
    pub show_hidden: bool,
}

impl FileManager {
    pub fn new(start_dir: PathBuf) -> Self {
        Self {
            current_dir: start_dir,
            history: Vec::new(),
            show_hidden: false,
        }
    }

    pub fn navigate_to(&mut self, path: PathBuf) -> Result<(), String> {
        if !path.is_dir() {
            return Err(format!("Not a directory: {}", path.display()));
        }
        let old = self.current_dir.clone();
        self.history.push(old);
        self.current_dir = path;
        Ok(())
    }

    pub fn go_back(&mut self) -> Option<PathBuf> {
        if let Some(prev) = self.history.pop() {
            let current = self.current_dir.clone();
            self.current_dir = prev;
            Some(current)
        } else {
            None
        }
    }

    pub fn list_entries(&self) -> Vec<FileEntry> {
        match std::fs::read_dir(&self.current_dir) {
            Ok(entries) => {
                let mut result: Vec<FileEntry> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| FileEntry::from_path(&e.path()))
                    .filter(|e| self.show_hidden || !e.is_hidden())
                    .collect();
                result.sort_by(|a, b| match (a.is_dir(), b.is_dir()) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                });
                result
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn filter_entries<'a>(&self, entries: &'a [FileEntry], query: &str) -> Vec<&'a FileEntry> {
        let q = query.to_lowercase();
        entries
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&q))
            .collect()
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
    }
}


#[cfg(test)]
mod folio_extra_tests {
    use super::*;

    #[test]
    fn test_human_size_formatting_aaa() {
        // Arrange
        let e_small = FileEntry {
            name: "small.txt".into(),
            path: PathBuf::from("/small.txt"),
            kind: EntryKind::File,
            size_bytes: 500,
            extension: Some("txt".into()),
        };
        let e_mb = FileEntry {
            name: "big.iso".into(),
            path: PathBuf::from("/big.iso"),
            kind: EntryKind::File,
            size_bytes: 10_485_760,
            extension: Some("iso".into()),
        };
        // Act & Assert
        assert_eq!(e_small.human_size(), "500 B");
        assert!(e_mb.human_size().contains("MB"));
    }

    #[test]
    fn test_directory_kind_aaa() {
        // Arrange
        let dir = FileEntry {
            name: "docs".into(),
            path: PathBuf::from("/docs"),
            kind: EntryKind::Directory,
            size_bytes: 4096,
            extension: None,
        };
        // Act & Assert
        assert!(dir.is_dir());
        assert!(!dir.is_hidden());
    }
}
