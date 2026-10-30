//! The Stroke dialog's right-to-left twins: only the direction is forced — the data and the
//! behaviour stay the ones the left-to-right tests pin.

use egui_kittest::kittest::Queryable;

use super::*;

/// The window the dialog is centred in: its width is what a mirrored row is checked against.
const WINDOW: f32 = 1280.0;

/// The harness pattern of the left-to-right tests: the app over one document.
fn harness() -> egui_kittest::Harness<'static, PhotocraftApp> {
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(WINDOW, 800.0))
        .with_max_steps(64)
        .build_eframe(|cc| {
            PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
            app.run("file.new", serde_json::json!({"width": 200, "height": 120})).unwrap();
            app
        });
    h.run_steps(6);
    h
}

/// The open Stroke dialog's Width field and Location dropdown, drawn under `dir`.
fn rows(dir: crate::text::direction::Dir) -> (egui::Rect, egui::Rect) {
    let _dir = crate::text::direction::direction_scope(dir);
    let mut h = harness();
    let ctx = h.ctx.clone();
    crate::menus::invoke(h.state_mut(), &ctx, COMMAND, serde_json::json!({})).unwrap();
    h.run_steps(3);
    (h.get_by_label(tl!("Width:")).rect(), h.get_by_value(tl!("Center")).rect())
}

/// The rows read from the right: the dialog is centred in the window, so a mirrored row's two
/// rects add up to the window's width, and each control moves to the other side of the centre.
#[test]
fn rtl_the_rows_read_from_the_right() {
    let (ltr_field, ltr_drop) = rows(crate::text::direction::Dir::Ltr);
    let (rtl_field, rtl_drop) = rows(crate::text::direction::Dir::Rtl);
    assert!(
        (ltr_field.center().x + rtl_field.center().x - WINDOW).abs() < 2.0,
        "the Width field mirrors: {ltr_field:?} / {rtl_field:?}"
    );
    assert!(
        (ltr_drop.center().x + rtl_drop.center().x - WINDOW).abs() < 2.0,
        "the Location dropdown mirrors: {ltr_drop:?} / {rtl_drop:?}"
    );
    // The control keeps the field's side of the caption: right of the centre in a left-to-right
    // row, left of it in a mirrored one.
    assert!(ltr_field.center().x > WINDOW / 2.0, "the field sits right of the centre: {ltr_field:?}");
    assert!(rtl_field.center().x < WINDOW / 2.0, "the field sits left of the centre: {rtl_field:?}");
    assert!(ltr_drop.center().x > WINDOW / 2.0, "the dropdown sits right of the centre: {ltr_drop:?}");
    assert!(rtl_drop.center().x < WINDOW / 2.0, "the dropdown sits left of the centre: {rtl_drop:?}");
}

/// The caption in its fixed column hugs the field it names: the box's right edge in a
/// left-to-right row, its left edge in a mirrored one.
#[test]
fn rtl_the_caption_hugs_its_field() {
    let r = egui::Rect::from_min_size(egui::pos2(10.0, 0.0), egui::vec2(112.0, 22.0));
    let (ltr_pos, ltr_align) = caption_anchor(r, false);
    assert!(matches!(ltr_align, Align2::RIGHT_CENTER), "the caption is right-aligned: {ltr_align:?}");
    assert!((ltr_pos - (r.right() - 6.0)).abs() < 0.01, "the caption sits at the right of its box: {ltr_pos:?}");
    let (rtl_pos, rtl_align) = caption_anchor(r, true);
    assert!(matches!(rtl_align, Align2::LEFT_CENTER), "the caption is left-aligned: {rtl_align:?}");
    assert!((rtl_pos - (r.left() + 6.0)).abs() < 0.01, "the caption sits at the left of its box: {rtl_pos:?}");
}
