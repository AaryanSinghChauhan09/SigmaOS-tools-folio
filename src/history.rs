//! Navigation history (back/forward stack).

/// Back/forward navigation history.
#[derive(Debug, Default)]
pub struct History {
    /// Visited paths.
    entries: Vec<String>,
    /// Current position (index into entries).
    pos: usize,
}

impl History {
    /// New empty history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a navigation to `path`.
    ///
    /// Truncates any forward entries (standard browser semantics).
    pub fn push(&mut self, path: impl Into<String>) {
        let path = path.into();
        self.entries.truncate(self.pos + 1);
        self.entries.push(path);
        self.pos = self.entries.len() - 1;
    }

    /// Whether back navigation is possible.
    pub fn can_back(&self) -> bool {
        self.pos > 0
    }

    /// Whether forward navigation is possible.
    pub fn can_forward(&self) -> bool {
        self.pos + 1 < self.entries.len()
    }

    /// Go back; returns new current path.
    pub fn back(&mut self) -> Option<&str> {
        if self.can_back() {
            self.pos -= 1;
            Some(self.entries[self.pos].as_str())
        } else {
            None
        }
    }

    /// Go forward; returns new current path.
    pub fn forward(&mut self) -> Option<&str> {
        if self.can_forward() {
            self.pos += 1;
            Some(self.entries[self.pos].as_str())
        } else {
            None
        }
    }

    /// Current path.
    pub fn current(&self) -> Option<&str> {
        self.entries.get(self.pos).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_back_forward() {
        let mut h = History::new();
        h.push("/a");
        h.push("/b");
        h.push("/c");
        assert_eq!(h.back(), Some("/b"));
        assert_eq!(h.back(), Some("/a"));
        assert_eq!(h.forward(), Some("/b"));
    }

    #[test]
    fn test_push_truncates_forward() {
        let mut h = History::new();
        h.push("/a");
        h.push("/b");
        h.back();
        h.push("/c");
        assert!(!h.can_forward());
        assert_eq!(h.current(), Some("/c"));
    }
}
