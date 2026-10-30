//! RTL twins for the Channel Options body: the blocks read from the right and the rows inside
//! them mirror, without the dialog changing width.

use egui_kittest::kittest::Queryable;
use serde_json::json;

use super::*;

/// The Channel Options dialog on the first alpha channel, rendered in the given direction.
fn channel_options(dir: crate::text::direction::Dir) -> egui_kittest::Harness<'static, PhotocraftApp> {
    let _dir = crate::text::direction::direction_scope(dir);
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
    app.run("file.new", json!({"width": 16, "height": 16})).unwrap();
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).build_ui_state(|ui, app| crate::dialogs::show(app, ui.ctx()), app);
    PhotocraftApp::setup_context(&h.ctx, crate::theme::ThemeKind::ALL[0]);
    open(h.state_mut(), &json!(0)).unwrap();
    h.run_steps(4);
    h
}

/// The dialog's own frame, as the host sized it.
fn frame(h: &egui_kittest::Harness<'static, PhotocraftApp>) -> egui::Rect {
    let id = h.state().ui.dialogs.last().unwrap().id;
    h.ctx.memory(|m| m.area_rect(egui::Id::unique(("dialog", id)))).expect("the dialog")
}

/// The body reads from the left in a left-to-right UI and from the right in a right-to-left one:
/// the section label starts at the reading start, the radios are indented from it and the name
/// field follows the caption column towards the reading end.
#[test]
fn rtl_the_channel_options_body_mirrors() {
    let ltr = channel_options(crate::text::direction::Dir::Ltr);
    let ltr_frame = frame(&ltr);
    let ltr_section = ltr.get_by_label("Color Indicates:").rect();
    let ltr_radio = ltr.get_by_label("Masked Areas").rect();
    let ltr_name = ltr.get_by_label("Name").rect();
    assert!(ltr_frame.left() - ltr_section.left() < 40.0, "the section label reads from the left: {ltr_section:?} in {ltr_frame:?}");
    assert!(ltr_radio.left() > ltr_section.left(), "the radios are indented from the left: {ltr_radio:?} vs {ltr_section:?}");
    assert!(ltr_name.left() - ltr_radio.left() > 70.0, "the name field follows the caption column: {ltr_name:?} vs {ltr_radio:?}");

    let rtl = channel_options(crate::text::direction::Dir::Rtl);
    let rtl_frame = frame(&rtl);
    let rtl_section = rtl.get_by_label("Color Indicates:").rect();
    let rtl_radio = rtl.get_by_label("Masked Areas").rect();
    let rtl_name = rtl.get_by_label("Name").rect();
    assert!(rtl_frame.right() - rtl_section.right() < 40.0, "the section label reads from the right: {rtl_section:?} in {rtl_frame:?}");
    assert!(rtl_radio.right() < rtl_section.right(), "the radios are indented from the right: {rtl_radio:?} vs {rtl_section:?}");
    assert!(rtl_radio.right() - rtl_name.right() > 70.0, "the name field follows the caption column: {rtl_name:?} vs {rtl_radio:?}");
}

/// The mirrored body keeps the dialog as wide as its left-to-right twin.
#[test]
fn rtl_the_channel_options_dialog_keeps_its_width() {
    let width = |dir| frame(&channel_options(dir)).width();
    let ltr = width(crate::text::direction::Dir::Ltr);
    let rtl = width(crate::text::direction::Dir::Rtl);
    assert!((ltr - rtl).abs() < 1.0, "LTR {ltr} vs RTL {rtl}");
}
