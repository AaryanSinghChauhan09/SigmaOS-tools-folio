//! Browser model — current directory, listing, selection, filtering.

use std::path::{Path, PathBuf};

use crate::entry::{Entry, EntryKind, natural_key};
use crate::history::History;
use crate::FolioError;

/// Sort mode for the file list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    /// Directories first, then natural name sort (folio default).
    #[default]
    DirsFirst,
    /// Name only, natural sort.
    Name,
    /// By modification time, newest first.
    Modified,
    /// By size, largest first.
    Size,
}

/// The file browser state.
#[derive(Debug)]
pub struct Browser {
    /// Current directory.
    pub current_dir: PathBuf,
    /// Visible entries (filtered + sorted).
    pub entries: Vec<Entry>,
    /// Selected entry index (None = nothing selected).
    pub selected: Option<usize>,
    /// Name filter substring (case-insensitive).
    pub filter: String,
    /// Sort mode.
    pub sort_mode: SortMode,
    /// Navigation history.
    pub history: History,
}

impl Browser {
    /// Open `dir` in the browser.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, FolioError> {
        let dir = dir.as_ref();
        if !dir.is_dir() {
            return Err(FolioError::NotDirectory(dir.display().to_string()));
        }
        let mut b = Self {
            current_dir: dir.to_path_buf(),
            entries: Vec::new(),
            selected: None,
            filter: String::new(),
            sort_mode: SortMode::default(),
            history: History::new(),
        };
        b.history.push(dir.display().to_string());
        b.refresh()?;
        Ok(b)
    }

    /// Re-read the current directory.
    pub fn refresh(&mut self) -> Result<(), FolioError> {
        let mut entries = Vec::new();
        for e in std::fs::read_dir(&self.current_dir)? {
            let e = e?;
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && name != "." && name != ".." {
                // Hidden files skipped unless filter shows them — folio
                // shows hidden only on demand; keep default hidden-off.
                continue;
            }
            let meta = e.metadata()?;
            let ft = meta.file_type();
            let kind = if ft.is_dir() {
                EntryKind::Directory
            } else if ft.is_symlink() {
                EntryKind::Symlink
            } else {
                EntryKind::File
            };
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            entries.push(Entry {
                name,
                path: e.path(),
                kind,
                size: if kind == EntryKind::File { meta.len() } else { 0 },
                modified,
            });
        }
        self.entries = entries;
        self.apply_filter_and_sort();
        Ok(())
    }

    fn apply_filter_and_sort(&mut self) {
        let f = self.filter.to_lowercase();
        if !f.is_empty() {
            self.entries.retain(|e| e.name.to_lowercase().contains(&f));
        }
        match self.sort_mode {
            SortMode::DirsFirst => {
                self.entries.sort_by(|a, b| {
                    let da = (a.kind != EntryKind::Directory) as u8;
                    let db = (b.kind != EntryKind::Directory) as u8;
                    da.cmp(&db).then_with(|| natural_key(&a.name).cmp(&natural_key(&b.name)))
                });
            }
            SortMode::Name => {
                self.entries.sort_by(|a, b| natural_key(&a.name).cmp(&natural_key(&b.name)));
            }
            SortMode::Modified => {
                self.entries.sort_by(|a, b| b.modified.cmp(&a.modified));
            }
            SortMode::Size => {
                self.entries.sort_by(|a, b| b.size.cmp(&a.size));
            }
        }
    }

    /// Navigate into a directory entry (by index).
    pub fn enter(&mut self, index: usize) -> Result<(), FolioError> {
        let entry = self.entries.get(index).ok_or_else(|| {
            FolioError::Navigation(format!("no entry at index {index}"))
        })?;
        if entry.kind != EntryKind::Directory {
            return Err(FolioError::Navigation(format!("{} is not a directory", entry.name)));
        }
        self.open_at(&entry.path.clone())?;
        Ok(())
    }

    /// Navigate to parent directory.
    pub fn parent(&mut self) -> Result<(), FolioError> {
        if let Some(p) = self.current_dir.parent() {
            self.open_at(p.to_path_buf())?;
        }
        Ok(())
    }

    /// Navigate to an arbitrary path (used by history/places).
    pub fn open_at(&mut self, dir: impl AsRef<Path>) -> Result<(), FolioError> {
        let dir = dir.as_ref();
        if !dir.is_dir() {
            return Err(FolioError::NotDirectory(dir.display().to_string()));
        }
        self.current_dir = dir.to_path_buf();
        self.history.push(dir.display().to_string());
        self.selected = None;
        self.refresh()
    }

    /// History back.
    pub fn back(&mut self) -> Result<(), FolioError> {
        if let Some(p) = self.history.back() {
            let p = p.to_string();
            self.current_dir = PathBuf::from(&p);
            self.refresh()?;
        }
        Ok(())
    }

    /// History forward.
    pub fn forward(&mut self) -> Result<(), FolioError> {
        if let Some(p) = self.history.forward() {
            let p = p.to_string();
            self.current_dir = PathBuf::from(&p);
            self.refresh()?;
        }
        Ok(())
    }

    /// Set the name filter.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.apply_filter_and_sort();
    }

    /// Select entry by index (keyboard navigation).
    pub fn select(&mut self, index: usize) {
        self.selected = if index < self.entries.len() { Some(index) } else { None };
    }

    /// Move selection up/down (wraps).
    pub fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let cur = self.selected.unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(self.entries.len() as isize) as usize;
        self.selected = Some(next);
    }

    /// Currently selected entry path.
    pub fn selected_path(&self) -> Option<&Path> {
        self.selected.and_then(|i| self.entries.get(i)).map(|e| e.path.as_path())
    }

    /// Path of entry at index.
    pub fn path_at(&self, index: usize) -> Option<&Path> {
        self.entries.get(index).map(|e| e.path.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("dir1")).unwrap();
        std::fs::create_dir_all(d.path().join("dir2")).unwrap();
        std::fs::write(d.path().join("file1.txt"), "a").unwrap();
        std::fs::write(d.path().join("file10.txt"), "b").unwrap();
        std::fs::write(d.path().join("file2.txt"), "c").unwrap();
        d
    }

    #[test]
    fn test_dirs_first_sort() {
        let d = fixture();
        let b = Browser::open(d.path()).unwrap();
        // First two entries are directories.
        assert_eq!(b.entries[0].kind, EntryKind::Directory);
        assert_eq!(b.entries[1].kind, EntryKind::Directory);
        // Files natural sorted: file1, file2, file10.
        let names: Vec<&str> = b.entries[2..].iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["file1.txt", "file2.txt", "file10.txt"]);
    }

    #[test]
    fn test_enter_and_parent() {
        let d = fixture();
        let mut b = Browser::open(d.path()).unwrap();
        // Enter first dir.
        b.enter(0).unwrap();
        assert!(b.current_dir.ends_with("dir1") || b.current_dir.ends_with("dir2"));
        b.parent().unwrap();
        assert_eq!(b.current_dir, d.path());
    }

    #[test]
    fn test_filter() {
        let d = fixture();
        let mut b = Browser::open(d.path()).unwrap();
        b.set_filter("file1");
        // Substring match hits file1.txt and file10.txt.
        assert_eq!(b.entries.len(), 2);
        assert!(b.entries.iter().all(|e| e.name.starts_with("file1")));
    }

    #[test]
    fn test_history_back_forward() {
        let d = fixture();
        let sub = d.path().join("dir1");
        let mut b = Browser::open(d.path()).unwrap();
        b.open_at(&sub).unwrap();
        b.back().unwrap();
        assert_eq!(b.current_dir, d.path());
        b.forward().unwrap();
        assert!(b.current_dir.ends_with("dir1") || b.current_dir.ends_with("dir2"));
    }
}
