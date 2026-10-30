//! The Source Files dialog mirrors in a right-to-left UI: the rows read from the right, the
//! file list keeps the reading start with its entries hugging it, and the modal keeps its width.

use egui::vec2;
use egui_kittest::kittest::Queryable;

use super::*;

/// The Load Files into Stack dialog over two files, drawn in `dir`: (harness, dialog id).
fn harness(dir: crate::text::direction::Dir) -> (egui_kittest::Harness<'static, PhotocraftApp>, u64) {
    let _dir = crate::text::direction::direction_scope(dir);
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    let id = open(&mut app);
    if let Some(d) = app.ui.dialog_mut(id) {
        add(&mut d.fields, ["/pictures/a.png".to_owned(), "/pictures/b.png".to_owned()]);
    }
    let dialogs = std::mem::take(&mut app.ui.dialogs);
    let mut h = egui_kittest::Harness::builder().with_size(vec2(1280.0, 800.0)).build_eframe(move |cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        let mut app = app;
        app.ui.dialogs = dialogs;
        app
    });
    h.run_steps(4);
    (h, id)
}

/// The geometry of one direction: the modal frame, the Use row, the file list and the column.
fn rects(dir: crate::text::direction::Dir) -> (egui::Rect, egui::Rect, egui::Rect, egui::Rect, egui::Rect, egui::Rect) {
    let (h, id) = harness(dir);
    let win = h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", id)))).expect("the dialog is on screen");
    (win, h.get_by_label("Use:").rect(), h.get_by_label("Use").rect(), h.get_by_label("a.png").rect(), h.get_by_label("b.png").rect(), h.get_by_label("Browse…").rect())
}

/// The dialog mirrors: the Use row reads from the right, the file list keeps the reading start
/// with its entries hugging it, and the modal's width does not change.
#[test]
fn rtl_the_source_files_read_from_the_right() {
    let (ltr_win, ltr_use, ltr_dropdown, ltr_a, ltr_b, ltr_browse) = rects(crate::text::direction::Dir::Ltr);
    assert!(ltr_use.right() < ltr_dropdown.left(), "LTR: the caption precedes the dropdown: {ltr_use:?} vs {ltr_dropdown:?}");
    assert!(ltr_a.right() < ltr_browse.left(), "LTR: the list keeps the reading start: {ltr_a:?} vs {ltr_browse:?}");
    assert!((ltr_a.left() - ltr_b.left()).abs() < 1.0, "LTR: the entries hug the list's start edge: {ltr_a:?} vs {ltr_b:?}");

    let (rtl_win, rtl_use, rtl_dropdown, rtl_a, rtl_b, rtl_browse) = rects(crate::text::direction::Dir::Rtl);
    assert!(rtl_use.left() > rtl_dropdown.right(), "RTL: the caption precedes the dropdown: {rtl_use:?} vs {rtl_dropdown:?}");
    assert!(rtl_a.left() > rtl_browse.right(), "RTL: the list keeps the reading start: {rtl_a:?} vs {rtl_browse:?}");
    assert!((rtl_a.right() - rtl_b.right()).abs() < 1.0, "RTL: the entries hug the list's start edge: {rtl_a:?} vs {rtl_b:?}");
    assert!((rtl_win.width() - ltr_win.width()).abs() < 1.0, "the modal keeps its width: {ltr_win:?} vs {rtl_win:?}");
}
