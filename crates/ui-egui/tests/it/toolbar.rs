//! Optimised PDF: tools pinned to the toolbar (`pdfcraft_ui_egui::toolbar`), in the real shell.

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;

fn harness() -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("form.pdf", None, include_bytes!("../data/form.pdf").to_vec()).unwrap();
        // The All tools list would show the category names too.
        app.set_option("left", "closed").unwrap();
        app
    });
    h.run_steps(4);
    h
}

/// The toolbar's pin button (its right-click menu holds the same) lists the tool categories;
/// hovering one shows all of its tools; clicking one pins it, and the pinned button runs it.
#[test]
fn right_clicking_the_toolbar_pins_a_sub_tool_that_then_runs() {
    let mut h = harness();
    assert!(h.state().toolbar_tools.is_empty());
    h.get_by_label("Add tools to the toolbar").click();
    h.run_steps(3);
    h.get_by_label("Organize pages").hover();
    h.run_steps(4);
    h.get_by_label("Rotate").click();
    h.run_steps(4);
    assert_eq!(h.state().toolbar_tools, ["page.rotate"], "Organize pages ▸ Rotate is pinned");
    // The pinned button runs the command on the open document.
    let id = h.state().views[0].id;
    assert_eq!(h.state().session.get(id).unwrap().info.pages[0].rotation, 0);
    h.get_by_label("Rotate pages clockwise").click();
    h.run_steps(4);
    assert_eq!(h.state().session.get(id).unwrap().info.pages[0].rotation, 90, "the pinned tool ran");
    // Kept in the settings.
    let saved = h.state().persist();
    let mut restored = PdfCraftApp::new();
    restored.restore(&saved);
    assert_eq!(restored.toolbar_tools, ["page.rotate"]);
}
