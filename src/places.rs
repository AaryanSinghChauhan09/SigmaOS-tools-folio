//! Places/bookmarks — sidebar locations.

use std::path::PathBuf;

/// A bookmarked location.
#[derive(Debug, Clone)]
pub struct Place {
    /// Display label.
    pub label: String,
    /// Path or URI.
    pub path: PathBuf,
    /// Icon name (freedesktop icon theme name, optional).
    pub icon: Option<String>,
}

/// User places list.
#[derive(Debug, Default)]
pub struct Places {
    /// Bookmarked places.
    pub items: Vec<Place>,
}

impl Places {
    /// Add a place.
    pub fn add(&mut self, label: impl Into<String>, path: impl Into<PathBuf>) {
        self.items.push(Place {
            label: label.into(),
            path: path.into(),
            icon: None,
        });
    }

    /// Find a place by label.
    pub fn get(&self, label: &str) -> Option<&Place> {
        self.items.iter().find(|p| p.label == label)
    }

    /// Remove a place by label.
    pub fn remove(&mut self, label: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|p| p.label != label);
        self.items.len() != before
    }
}

/// Default places (home, root, common mounts) — upstream shows these
/// plus XDG user dirs.
pub fn default_places() -> Places {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let mut p = Places::default();
    p.add("Home", PathBuf::from(&home));
    p.add("Root", PathBuf::from("/"));
    for (label, sub) in [
        ("Desktop", "Desktop"),
        ("Documents", "Documents"),
        ("Downloads", "Downloads"),
        ("Music", "Music"),
        ("Pictures", "Pictures"),
        ("Videos", "Videos"),
    ] {
        let path = PathBuf::from(&home).join(sub);
        if path.exists() {
            p.add(label, path);
        }
    }
    p
}
