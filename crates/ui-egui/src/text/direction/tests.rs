//! Right-to-left shell layout: the direction rules, and where the mirrored chrome lands in a
//! running shell. The shell tests force the direction with `direction_scope`, so they don't
//! depend on a right-to-left language being selected.

use super::*;
use crate::PhotocraftApp;
use crate::i18n::{self, Lang};
use egui_kittest::{Harness, kittest::Queryable};
use serde_json::json;

#[test]
fn direction_follows_the_language_unless_forced() {
    let _lang = i18n::language_scope(Lang::EN);
    let forced = direction_scope(Dir::Rtl);
    assert!(is_rtl());
    {
        let _inner = direction_scope(Dir::Ltr);
        assert!(!is_rtl());
    }
    assert!(is_rtl(), "the inner scope restores the outer one");
    drop(forced);
    assert_eq!(FORCED.get(), None);
}

#[test]
fn sides_and_rows_mirror() {
    {
        let _ltr = direction_scope(Dir::Ltr);
        assert_eq!(row().main_dir(), egui::Direction::LeftToRight);
        assert_eq!(trailing().main_dir(), egui::Direction::RightToLeft);
        assert_eq!(below(), egui::RectAlign::BOTTOM_START);
    }
    let _rtl = direction_scope(Dir::Rtl);
    assert_eq!(row().main_dir(), egui::Direction::RightToLeft);
    assert_eq!(trailing().main_dir(), egui::Direction::LeftToRight);
    assert_eq!(below(), egui::RectAlign::BOTTOM_END);
}

#[test]
fn body_and_footer_layouts_mirror() {
    {
        let _ltr = direction_scope(Dir::Ltr);
        assert_eq!(body_layout().main_dir(), egui::Direction::TopDown);
        assert_eq!(body_layout().cross_align(), Align::Min);
        assert_eq!(trailing_min().main_dir(), egui::Direction::RightToLeft);
        assert_eq!(trailing_min().cross_align(), Align::Min);
        assert_eq!(horizontal_nav_keys(), (egui::Key::ArrowRight, egui::Key::ArrowLeft));
    }
    let _rtl = direction_scope(Dir::Rtl);
    // Vertical + `Align::Max` is what makes egui mirror the nested rows of a body.
    assert_eq!(body_layout().main_dir(), egui::Direction::TopDown);
    assert_eq!(body_layout().cross_align(), Align::Max);
    assert_eq!(trailing_min().main_dir(), egui::Direction::LeftToRight);
    assert_eq!(trailing_min().cross_align(), Align::Min);
    assert_eq!(horizontal_nav_keys(), (egui::Key::ArrowLeft, egui::Key::ArrowRight));
}

#[test]
fn env_values_parse_strictly() {
    assert_eq!(parse_dir("rtl"), Some(Dir::Rtl));
    assert_eq!(parse_dir(" RTL "), Some(Dir::Rtl));
    assert_eq!(parse_dir("ltr"), Some(Dir::Ltr));
    assert_eq!(parse_dir(""), None);
    assert_eq!(parse_dir("ar"), None);
}

/// The egui direction flag ([`crate::theme::set_direction`]) must follow the direction scope so
/// every egui-owned widget (submenus, popups, scrollbars, window chrome) mirrors with it.
#[test]
fn right_to_left_flag_follows_direction() {
    let _lang = i18n::language_scope(Lang::EN);
    for (dir, want) in [(Dir::Ltr, false), (Dir::Rtl, true)] {
        let _dir = direction_scope(dir);
        let h = harness();
        assert_eq!(h.ctx.global_style().right_to_left, want, "{dir:?}");
    }
}

/// A committed language change flips the flag in the same frame (i18n::sync_context), in both
/// egui theme branches so an appearance switch cannot revert the direction.
#[test]
fn the_flag_follows_a_live_language_switch() {
    let Some(_) = photocraft_text::craft_fonts::CRAFT_FONTS.iter().find(|f| f.family == "Noto Sans Arabic") else {
        eprintln!("skipping: built without craft-fonts (set CRAFT_FONTS_DIR to a craft-fonts checkout)");
        return;
    };
    let _lang = i18n::language_scope(Lang::EN);
    let mut h = harness();
    assert!(!h.ctx.global_style().right_to_left, "English starts left-to-right");
    h.state_mut().run("prefs.set", json!({"path": "interface.language", "value": "ar"})).expect("language");
    h.run_steps(4);
    assert!(h.ctx.global_style().right_to_left, "switching to Arabic flips the flag");
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        assert!(h.ctx.style_of(theme).right_to_left, "{theme:?} branch");
    }
    h.state_mut().run("prefs.set", json!({"path": "interface.language", "value": "en"})).expect("language");
    h.run_steps(4);
    assert!(!h.ctx.global_style().right_to_left, "switching back restores left-to-right");
}

fn harness() -> Harness<'static, PhotocraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1200.0, 800.0)).with_max_steps(64).build_eframe(|cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, crate::theme::ThemeKind::ProMedium);
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"width": 64, "height": 64})).expect("new document");
        app
    });
    h.run_steps(4);
    h
}

/// Centre x of the widget labelled `label` (the first, when several are).
fn x_of(h: &Harness<'static, PhotocraftApp>, label: &str) -> f32 {
    h.query_all_by_label(label).next().unwrap_or_else(|| panic!("`{label}` drawn")).rect().center().x
}

/// Left x of the text `text` where it is painted (for widgets without an accessible label).
fn text_x(h: &Harness<'static, PhotocraftApp>, text: &str) -> f32 {
    fn find(shape: &egui::Shape, text: &str) -> Option<f32> {
        match shape {
            egui::Shape::Text(t) if t.galley.job.text == text => Some(t.pos.x),
            egui::Shape::Vec(shapes) => shapes.iter().find_map(|s| find(s, text)),
            _ => None,
        }
    }
    h.output().shapes.iter().find_map(|s| find(&s.shape, text)).unwrap_or_else(|| panic!("`{text}` painted"))
}

fn dock_x(h: &Harness<'static, PhotocraftApp>) -> f32 {
    crate::panel_docking::probe::panel_rect(&h.ctx, "layers").expect("Layers docked").center().x
}

#[test]
fn the_shell_mirrors_in_a_right_to_left_ui() {
    let _lang = i18n::language_scope(Lang::EN);
    for dir in [Dir::Ltr, Dir::Rtl] {
        let _dir = direction_scope(dir);
        let h = harness();
        let mid = 600.0;
        let (file, help, toolbar, dock, workspace) = (x_of(&h, "File"), x_of(&h, "Help"), x_of(&h, "Toolbar"), dock_x(&h), text_x(&h, "Essentials"));
        if dir == Dir::Rtl {
            assert!(file > help, "File starts the menu bar at the right: {file} vs {help}");
            assert!(toolbar > mid && dock < mid, "toolbox right, dock left: {toolbar}, {dock}");
            assert!(workspace < help, "the workspace switcher is at the far (left) end: {workspace}");
        } else {
            assert!(file < help, "{file} vs {help}");
            assert!(toolbar < mid && dock > mid, "{toolbar}, {dock}");
            assert!(workspace > help, "{workspace}");
        }
    }
}

#[test]
fn arrow_keys_step_through_the_menu_bar_in_screen_order() {
    let _lang = i18n::language_scope(Lang::EN);
    for (dir, towards_edit) in [(Dir::Ltr, egui::Key::ArrowRight), (Dir::Rtl, egui::Key::ArrowLeft)] {
        let _dir = direction_scope(dir);
        let mut h = harness();
        h.get_by_label("File").click();
        h.run_steps(4);
        assert!(h.query_by_label_contains("Undo").is_none(), "File is open, not Edit");
        // Edit is File's neighbour on screen: to its right, or to its left in a right-to-left UI.
        h.key_press(towards_edit);
        h.run_steps(4);
        assert!(h.query_by_label_contains("Undo").is_some(), "{dir:?}: {towards_edit:?} opens Edit");
    }
}

#[test]
fn menus_open_under_the_title_s_reading_start_edge() {
    let _lang = i18n::language_scope(Lang::EN);
    let _dir = direction_scope(Dir::Rtl);
    let mut h = harness();
    let edit = h.get_by_label("Edit").rect();
    h.get_by_label("Edit").click();
    h.run_steps(4);
    let undo = h.get_by_label_contains("Undo").rect();
    assert!(undo.right() <= edit.right() + 1.0 && undo.right() > edit.right() - 40.0, "the menu hangs left from Edit's right edge: {undo:?} {edit:?}");
}

#[test]
fn hand_painted_x_helpers_mirror_the_edges() {
    let rect = egui::Rect::from_min_size(egui::pos2(100.0, 40.0), egui::vec2(300.0, 22.0));
    {
        let _ltr = direction_scope(Dir::Ltr);
        assert_eq!(step(), 1.0);
        assert_eq!(start_x(rect, 6.0), 106.0);
        assert_eq!(start_x_w(rect, 6.0, 22.0), 106.0);
        assert_eq!(end_x(rect, 10.0), 390.0);
    }
    let _rtl = direction_scope(Dir::Rtl);
    assert_eq!(step(), -1.0);
    assert_eq!(start_x(rect, 6.0), 394.0);
    assert_eq!(start_x_w(rect, 6.0, 22.0), 372.0);
    assert_eq!(end_x(rect, 10.0), 110.0);
}

#[test]
fn hand_painted_x_helpers_mirror_labels_aligns_and_glyphs() {
    let rect = egui::Rect::from_min_size(egui::pos2(100.0, 40.0), egui::vec2(300.0, 22.0));
    {
        let _ltr = direction_scope(Dir::Ltr);
        assert_eq!(after_x(rect, 10.0, 30.0), 410.0);
        assert_eq!(start_align(), egui::Align2::LEFT_CENTER);
        assert_eq!(end_align(), egui::Align2::RIGHT_CENTER);
        assert!(start_side(105.0, 110.0));
        assert!(!start_side(115.0, 110.0));
        assert_eq!(glyph_x(102.0, 106.0), 102.0);
    }
    let _rtl = direction_scope(Dir::Rtl);
    assert_eq!(after_x(rect, 10.0, 30.0), 60.0);
    assert_eq!(after_x(rect, 10.0, 0.0), 90.0);
    assert_eq!(start_align(), egui::Align2::RIGHT_CENTER);
    assert_eq!(end_align(), egui::Align2::LEFT_CENTER);
    assert!(start_side(115.0, 110.0));
    assert!(!start_side(105.0, 110.0));
    assert_eq!(glyph_x(102.0, 106.0), 110.0);
}

/// `popup_body` re-scopes the body right-to-left under RTL — justified at `Align::Min`, pinned to
/// the parent `max_rect` — and keeps the popup's own layout left-to-right.
#[test]
fn popup_body_mirrors_only_under_rtl() {
    for (dir, mirrored) in [(Dir::Ltr, false), (Dir::Rtl, true)] {
        let _dir = direction_scope(dir);
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|s| s.right_to_left = dir == Dir::Rtl);
        let mut inner = None;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            ui.allocate_ui_with_layout(egui::vec2(300.0, 200.0), Layout::left_to_right(Align::Center), |ui| {
                popup_body(ui, |ui| {
                    inner = Some((ui.layout().main_dir(), ui.layout().cross_align(), ui.max_rect().width()));
                    ui.label("Undo");
                });
            });
        });
        out.textures_delta.clear();
        let (main_dir, cross, max_w) = inner.expect("popup_body ran its contents");
        if mirrored {
            assert_eq!(main_dir, egui::Direction::TopDown, "{dir:?}");
            assert_eq!(cross, Align::Min, "{dir:?}: the cross align stays Min (Max ballooned popup width)");
            assert_eq!(max_w, 300.0, "{dir:?}: the scope is pinned to the parent max_rect");
        } else {
            assert_eq!(main_dir, egui::Direction::LeftToRight, "{dir:?}: LTR keeps the popup's own layout");
        }
    }
}

/// The RTL `popup_body` scope must not balloon the body's `min_rect` past the pinned width: the
/// cross align stays `Min`, so the popup measures the rows' width, not the full area width.
#[test]
fn popup_body_rtl_min_rect_stays_inside_the_pinned_width() {
    for dir in [Dir::Ltr, Dir::Rtl] {
        let _dir = direction_scope(dir);
        let ctx = egui::Context::default();
        ctx.all_styles_mut(|s| s.right_to_left = dir == Dir::Rtl);
        let mut width = 0.0;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            ui.allocate_ui_with_layout(egui::vec2(300.0, 200.0), Layout::top_down_justified(Align::Min), |ui| {
                popup_body(ui, |ui| {
                    let _ = ui.button("Undo");
                    let _ = ui.button("Preferences");
                });
                width = ui.min_rect().width();
            });
        });
        out.textures_delta.clear();
        assert!(width <= 300.5, "{dir:?}: popup_body ballooned its min_rect to {width} inside a 300-wide scope");
    }
}

/// Left-to-right `popup_body` is a pass-through: wrapping the same contents measures identically
/// to calling them directly.
#[test]
fn popup_body_ltr_matches_a_direct_call() {
    let _ltr = direction_scope(Dir::Ltr);
    let ctx = egui::Context::default();
    let mut widths = [0.0_f32; 2];
    let mut out = ctx.run_ui(Default::default(), |ui| {
        ui.allocate_ui_with_layout(egui::vec2(300.0, 200.0), Layout::top_down_justified(Align::Min), |ui| {
            let _ = ui.button("Undo");
            widths[0] = ui.min_rect().width();
        });
        ui.allocate_ui_with_layout(egui::vec2(300.0, 200.0), Layout::top_down_justified(Align::Min), |ui| {
            popup_body(ui, |ui| {
                let _ = ui.button("Undo");
            });
            widths[1] = ui.min_rect().width();
        });
    });
    out.textures_delta.clear();
    assert_eq!(widths[0], widths[1], "LTR: popup_body changed the body's measured width");
}
