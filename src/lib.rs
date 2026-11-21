//! SigmaOS tools-folio component.
//!
//! Rust implementation of `omacom/folio`: Omarchy's keyboard-first file
//! manager (QML/Quickshell upstream).
//!
//! This port implements the file-browser domain model the UI binds to:
//! directory listing with dirs-first natural sort, selection, navigation
//! history (back/forward), bookmarks/places, and name filtering — the
//! logic upstream keeps in QML models.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod browser;
pub mod entry;
pub mod error;
pub mod history;
pub mod places;

pub use browser::{Browser, SortMode};
pub use entry::{Entry, EntryKind};
pub use error::FolioError;
pub use history::History;
pub use places::{Places, default_places};
