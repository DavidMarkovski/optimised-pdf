//! Pinned folders: the setting surviving a restart, and the limit on how many can be pinned.
//! Optimised PDF's Home doesn't list pinned folders (Home is Open file and Recent only), so the
//! tests that clicked through that section were removed; the setting and the control channel
//! (`control::control_collapses_and_expands_a_pinned_folder`) remain covered.

use std::path::{Path, PathBuf};

use pdfcraft_ui_egui::PdfCraftApp;
use pdfcraft_ui_egui::folders_ui::MAX_PINNED;

/// A fresh, empty folder under the temp dir, removed when dropped.
struct TempFolder(PathBuf);

impl TempFolder {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("pdfcraft-pinned-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn pinned_folders_survive_a_restart() {
    let dir = TempFolder::new("restart");
    let folder = dir.path();
    let mut a = PdfCraftApp::new();
    a.set_option("language", "en").unwrap();
    assert!(a.pin_folder(&folder));
    let mut b = PdfCraftApp::new();
    b.set_option("language", "en").unwrap();
    b.restore(&a.persist());
    assert_eq!(b.pinned.folders, [folder]);
    b.restore("{\"pinned_folders\": [5, \"relative/path\"]}");
    assert!(b.pinned.folders.is_empty(), "only absolute folder paths are restored");
    b.restore("{not json");
    assert!(b.pinned.folders.is_empty());
}

#[test]
fn pinning_is_limited_and_never_duplicates() {
    let mut app = PdfCraftApp::new();
    app.set_option("language", "en").unwrap();
    assert!(!app.pin_folder("relative/scans"), "a relative path is refused");
    let root = if cfg!(windows) { "C:\\" } else { "/" };
    let folders: Vec<String> = (0..MAX_PINNED).map(|i| Path::new(root).join(format!("pinned-{i}")).to_string_lossy().into_owned()).collect();
    for f in &folders {
        assert!(app.pin_folder(f));
    }
    assert!(app.pin_folder(&folders[0]), "pinning again is fine");
    assert_eq!(app.pinned.folders.len(), MAX_PINNED, "and adds nothing");
    assert!(!app.pin_folder(&Path::new(root).join("one-too-many").to_string_lossy()));
    assert!(app.toast.as_ref().is_some_and(|(m, _)| m.contains("Unpin one first")), "the user is told");
}
