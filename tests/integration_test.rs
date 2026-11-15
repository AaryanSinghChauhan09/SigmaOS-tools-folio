use sigmaos_tools_folio::{FileEntry, FileManager};
use std::path::PathBuf;

#[test]
fn test_navigate_and_list_aaa() {
    // Arrange
    let mut fm = FileManager::new(PathBuf::from("/tmp"));
    // Act
    fm.navigate_to(PathBuf::from("/tmp")).unwrap();
    // Assert
    assert_eq!(fm.current_dir, PathBuf::from("/tmp"));
}

#[test]
fn test_history_navigation_aaa() {
    // Arrange
    let mut fm = FileManager::new(PathBuf::from("/tmp"));
    fm.navigate_to(PathBuf::from("/")).unwrap();
    // Act
    let went_back = fm.go_back();
    // Assert
    assert!(went_back.is_some());
    assert_eq!(fm.current_dir, PathBuf::from("/tmp"));
}

#[test]
fn test_hidden_file_detection_aaa() {
    // Arrange
    let entry = FileEntry {
        name: ".hidden".into(),
        path: PathBuf::from("/.hidden"),
        kind: sigmaos_tools_folio::EntryKind::File,
        size_bytes: 0,
        extension: None,
    };
    // Act & Assert
    assert!(entry.is_hidden());
}
