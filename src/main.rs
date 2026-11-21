//! folio — file manager for SigmaOS (CLI front-end over the browser model).

use std::env;
use std::path::PathBuf;

use sigmaos_tools_folio::{Browser, SortMode};

fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"))
    });
    let mut browser = match Browser::open(&dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("folio: {e}");
            std::process::exit(1);
        }
    };
    browser.sort_mode = SortMode::DirsFirst;
    println!("folio — {}", browser.current_dir.display());
    for (i, e) in browser.entries.iter().enumerate() {
        let marker = if Some(i) == browser.selected { ">" } else { " " };
        let kind = match e.kind {
            sigmaos_tools_folio::EntryKind::Directory => "d",
            sigmaos_tools_folio::EntryKind::Symlink => "l",
            sigmaos_tools_folio::EntryKind::File => "-",
        };
        println!("{}{} {:>10} {}", marker, kind, e.size, e.name);
    }
    println!("{} entries", browser.entries.len());
}
