//! Directory entry model.

use std::path::PathBuf;

/// Entry kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
    /// Symlink.
    Symlink,
}

/// A directory entry.
#[derive(Debug, Clone)]
pub struct Entry {
    /// File name.
    pub name: String,
    /// Full path.
    pub path: PathBuf,
    /// Entry kind.
    pub kind: EntryKind,
    /// Size in bytes (files only; dirs report 0).
    pub size: u64,
    /// Modification time (unix secs).
    pub modified: i64,
}

impl Entry {
    /// Whether this is `..` parent entry.
    pub fn is_parent(&self) -> bool {
        self.name == ".."
    }

    /// Whether this is `.` current entry.
    pub fn is_current(&self) -> bool {
        self.name == "."
    }
}

/// Natural-sort key: split into digit/non-digit runs for `file2` < `file10`.
pub fn natural_key(name: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut cur_digit = false;
    for c in name.chars() {
        let d = c.is_ascii_digit();
        if !cur.is_empty() && d != cur_digit {
            parts.push(std::mem::take(&mut cur));
        }
        cur_digit = d;
        cur.extend(c.to_lowercase());
    }
    if !cur.is_empty() {
        parts.push(cur);
    }
    // Zero-pad digit runs so numeric segments compare numerically.
    parts
        .into_iter()
        .map(|p| {
            if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() {
                format!("{:020}", p.parse::<u64>().unwrap_or(0))
            } else {
                p
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort() {
        let mut names = vec!["file10", "file2", "file1"];
        names.sort_by_key(|n| natural_key(n));
        assert_eq!(names, vec!["file1", "file2", "file10"]);
    }
}
