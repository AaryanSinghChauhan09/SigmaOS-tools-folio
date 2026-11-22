//! Terminal UI renderer — file browser as an ANSI listing.
//!
//! Upstream folio is a Quickshell file manager; this port renders the
//! `Browser` state to terminal lines: path bar, entries with kind
//! icons, selection highlight, and footer with counts.

use crate::browser::Browser;
use crate::entry::{Entry, EntryKind};

/// Render options.
#[derive(Debug, Clone)]
pub struct TuiOpts {
    /// Terminal width.
    pub width: usize,
    /// Terminal height.
    pub height: usize,
    /// Use ANSI colors.
    pub color: bool,
}

impl Default for TuiOpts {
    fn default() -> Self {
        Self { width: 80, height: 24, color: true }
    }
}

/// Render the browser to terminal lines.
pub fn render_browser(browser: &Browser, opts: &TuiOpts) -> Vec<String> {
    let mut lines = Vec::new();
    // Path bar.
    lines.push(format!(" {}", path_bar(browser, opts)));
    lines.push("─".repeat(opts.width.saturating_sub(1)));
    // Entries.
    for (i, e) in browser.entries.iter().enumerate() {
        let selected = browser.selected == Some(i);
        lines.push(render_entry(e, selected, opts));
        if lines.len() >= opts.height - 2 {
            break;
        }
    }
    // Footer.
    while lines.len() < opts.height - 1 {
        lines.push(String::new());
    }
    lines.push(footer(browser, opts));
    lines.truncate(opts.height);
    lines
}

fn path_bar(browser: &Browser, opts: &TuiOpts) -> String {
    let path = browser.current_dir.display().to_string();
    let filter = if browser.filter.is_empty() {
        String::new()
    } else {
        format!("  /{}", browser.filter)
    };
    let s = format!(" {}{}", path, filter);
    if opts.color {
        format!("\x1b[1m\x1b[36m{}\x1b[0m", s)
    } else {
        s
    }
}

fn render_entry(e: &Entry, selected: bool, opts: &TuiOpts) -> String {
    let (icon, color) = match e.kind {
        EntryKind::Directory => ("📁", "\x1b[34m"), // blue
        EntryKind::File => ("📄", "\x1b[37m"),     // white
        EntryKind::Symlink => ("🔗", "\x1b[36m"),  // cyan
    };
    let name = if e.name == ".." { "..".to_string() } else { e.name.clone() };
    let size = if e.kind == EntryKind::File {
        format!(" {:>10}", human_size(e.size))
    } else {
        String::new()
    };
    let row = format!(" {} {}{}{}", icon, name, size, " ".repeat(opts.width.saturating_sub(20)));
    if selected {
        if opts.color {
            format!("\x1b[7m{}\x1b[0m", row)
        } else {
            format!(">{}", row)
        }
    } else if opts.color {
        format!("{}{}\x1b[0m", color, row)
    } else {
        row
    }
}

fn footer(browser: &Browser, opts: &TuiOpts) -> String {
    let total = browser.entries.len();
    let sel = browser.selected.map(|i| i + 1).unwrap_or(0);
    let text = format!(" {}/{}  {}", sel, total, browser.current_dir.display());
    let pad = opts.width.saturating_sub(text.len());
    if opts.color {
        format!("{}\x1b[2m{}\x1b[0m", " ".repeat(pad), text)
    } else {
        format!("{}{}", " ".repeat(pad), text)
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut v = bytes as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u == 0 {
        format!("{}B", bytes)
    } else {
        format!("{:.1}{}", v, UNITS[u])
    }
}

/// Handle a key event for the browser (returns true if handled).
pub fn handle_key(browser: &mut Browser, key: &str) -> bool {
    match key {
        "j" | "down" => { browser.move_selection(1); true }
        "k" | "up" => { browser.move_selection(-1); true }
        "enter" => { browser.enter(browser.selected.unwrap_or(0)).is_ok() }
        "backspace" | "h" => { browser.parent().is_ok() }
        "l" | "right" => { browser.back().is_ok() }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{Entry, EntryKind};
    use std::path::PathBuf;

    fn browser() -> Browser {
        Browser {
            current_dir: PathBuf::from("/tmp"),
            entries: vec![
                Entry { name: "dir".into(), path: PathBuf::from("/tmp/dir"), kind: EntryKind::Directory, size: 0, modified: 0 },
                Entry { name: "file.txt".into(), path: PathBuf::from("/tmp/file.txt"), kind: EntryKind::File, size: 2048, modified: 0 },
            ],
            selected: Some(0),
            filter: String::new(),
            sort_mode: crate::browser::SortMode::Name,
            history: crate::history::History::new(),
        }
    }

    #[test]
    fn test_render() {
        let b = browser();
        let lines = render_browser(&b, &TuiOpts::default());
        assert!(lines.iter().any(|l| l.contains("/tmp")));
        assert!(lines.iter().any(|l| l.contains("file.txt")));
        assert!(lines.iter().any(|l| l.contains("dir")));
    }

    #[test]
    fn test_key_nav() {
        let mut b = browser();
        assert!(handle_key(&mut b, "j"));
        assert_eq!(b.selected, Some(1));
        assert!(handle_key(&mut b, "k"));
        assert_eq!(b.selected, Some(0));
    }
}
