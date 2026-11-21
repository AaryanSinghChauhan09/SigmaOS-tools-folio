//! Integration tests for sigmaos-tools-folio.

use sigmaos_tools_folio::{Browser, History, default_places};


fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("docs")).unwrap();
    std::fs::write(d.path().join("a.txt"), "x").unwrap();
    std::fs::write(d.path().join("b.txt"), "y").unwrap();
    d
}

#[test]
fn test_browser_lists_directory() {
    let d = fixture();
    let b = Browser::open(d.path()).unwrap();
    assert!(b.entries.iter().any(|e| e.name == "a.txt"));
    assert!(b.entries.iter().any(|e| e.name == "docs"));
}

#[test]
fn test_places_default() {
    let p = default_places();
    assert!(p.get("Home").is_some());
    assert!(p.get("Root").is_some());
}

#[test]
fn test_history_nav() {
    let d = fixture();
    let mut h = History::new();
    h.push("/a"); h.push("/b"); h.push("/c");
    assert!(h.can_back());
    assert_eq!(h.back(), Some("/b"));
    assert_eq!(h.forward(), Some("/c"));
}
