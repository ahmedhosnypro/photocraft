//! Photoshop's New Group dialog (Name, Color, Mode, Opacity), opened by Layer › New › Group… and
//! Group from Layers…, the Layers panel menu's New Group… / New Group from Layers… and ⌥-click on
//! the panel's New Group button. OK runs `layer.new.group` / `layer.new.groupFromLayers` with the
//! options as one history step; explicit params from Actions, CLI and MCP run the engine directly.

use egui::{Align2, Sense, pos2, vec2};
use photocraft_color::BlendMode;
use photocraft_doc::LabelColor;
use serde_json::{Map, Value, json};

use crate::PhotocraftApp;
use crate::theme::Tokens;

pub const NEW_GROUP: &str = "layer.new.group";
pub const FROM_LAYERS: &str = "layer.new.groupFromLayers";
const MARK: &str = "__newGroup";
const KEYS: [&str; 4] = ["name", "color", "mode", "opacity"];

pub fn owns(fields: &Map<String, Value>) -> bool {
    fields.contains_key(MARK)
}

/// Whether the dialog fronts command `id`.
pub fn fronts(id: &str) -> bool {
    id == NEW_GROUP || id == FROM_LAYERS
}

/// Photoshop's defaults: the next "Group N", no colour, Pass Through, 100 %. Unlike Stroke the
/// choices are not remembered, as in Photoshop.
pub fn fields(app: &PhotocraftApp, command: &str) -> Map<String, Value> {
    let name = app.session.active().map(|st| st.doc.next_layer_name("Group")).unwrap_or_else(|| "Group 1".into());
    let mut f = Map::new();
    f.insert(MARK.into(), json!(true));
    f.insert("__command".into(), json!(if command == FROM_LAYERS { FROM_LAYERS } else { NEW_GROUP }));
    // The window title, as in Photoshop.
    f.insert("__label".into(), json!(if command == FROM_LAYERS { "New Group from Layers…" } else { "New Group…" }));
    f.insert("name".into(), json!(name));
    f.insert("color".into(), json!(LabelColor::None.id()));
    f.insert("mode".into(), json!(crate::dialog_blend_ui::mode_key(BlendMode::PassThrough)));
    f.insert("opacity".into(), json!(100.0));
    f
}

pub fn open(app: &mut PhotocraftApp, command: &str) -> u64 {
    let f = fields(app, command);
    app.ui.open_dialog(crate::state::DialogKind::Command, f)
}

/// Only the New Group params are passed to the engine; private UI markers are ignored.
pub fn params(f: &Map<String, Value>) -> Value {
    Value::Object(KEYS.iter().filter_map(|k| f.get(*k).map(|v| (k.to_string(), v.clone()))).collect())
}

pub fn confirm(app: &mut PhotocraftApp, f: &Map<String, Value>) -> Result<Value, String> {
    let command = f.get("__command").and_then(Value::as_str).filter(|c| fronts(c)).unwrap_or(NEW_GROUP).to_string();
    app.run(&command, params(f))
}

fn label(ui: &mut egui::Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(64.0, 22.0), Sense::hover());
    // The caption hugs the control beside it: the box's right edge in a left-to-right row, its
    // left edge in a mirrored one.
    let (x, anchor) = if crate::direction::is_rtl() { (r.left() + 6.0, Align2::LEFT_CENTER) } else { (r.right() - 6.0, Align2::RIGHT_CENTER) };
    ui.painter().text(pos2(x, r.center().y), anchor, text, egui::FontId::proportional(12.0), t.text_dim);
}

/// The label colour's swatch beside its menu: a crossed box for No Color, as in the layer menu.
fn swatch(ui: &mut egui::Ui, color: LabelColor) {
    let t = Tokens::get(ui.ctx());
    let (r, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
    let p = ui.painter();
    if let Some((fill, _)) = t.layer_label_colors(color) {
        p.rect_filled(r, t.radius_sm, fill);
    } else {
        let stroke = egui::Stroke::new(1.0, t.text_dim);
        p.rect_stroke(r, t.radius_sm, stroke, egui::StrokeKind::Inside);
        let x = r.shrink(4.0);
        p.line_segment([x.left_top(), x.right_bottom()], stroke);
        p.line_segment([x.right_top(), x.left_bottom()], stroke);
    }
}

pub fn body(ui: &mut egui::Ui, f: &mut Map<String, Value>) {
    ui.spacing_mut().item_spacing.y = 8.0;
    // One direction-aware body: in a left-to-right UI the layout is egui's default, so nothing
    // moves; in a right-to-left one the rows read from the right and every field follows its
    // caption.
    ui.with_layout(crate::direction::body_layout(), |ui| body_fields(ui, f));
}

/// The New Group dialog's fields.
fn body_fields(ui: &mut egui::Ui, f: &mut Map<String, Value>) {
    ui.horizontal(|ui| {
        label(ui, tl!("Name:"));
        let mut name = f.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
        let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(252.0));
        if r.changed() {
            f.insert("name".into(), json!(name));
        }
        crate::field_tab::register(ui.ctx(), r.id);
        // Photoshop opens the dialog with the name selected, so typing replaces it.
        if !f.contains_key("__focused") {
            f.insert("__focused".into(), json!(true));
            crate::field_tab::focus(ui.ctx(), r.id);
        }
        if r.gained_focus() {
            crate::field_tab::select_all(ui.ctx(), r.id, &name);
        }
    });
    ui.horizontal(|ui| {
        label(ui, tl!("Color:"));
        let mut color = f.get("color").and_then(Value::as_str).and_then(LabelColor::from_id).unwrap_or(LabelColor::None);
        let options: Vec<(LabelColor, &str)> = LabelColor::ALL.iter().map(|c| (*c, crate::layer_menu_ui::color_name(*c))).collect();
        if crate::widgets::dropdown(ui, "new-group-color", tl!("Color"), &mut color, &options, 130.0) {
            f.insert("color".into(), json!(color.id()));
        }
        swatch(ui, color);
    });
    ui.horizontal(|ui| {
        label(ui, tl!("Mode:"));
        let mut mode = f.get("mode").and_then(Value::as_str).and_then(photocraft_engine::commands::blend_from_str).unwrap_or(BlendMode::PassThrough);
        let options = crate::panels::blend_options(true, mode);
        if crate::widgets::dropdown(ui, "new-group-mode", tl!("Mode"), &mut mode, &options, 130.0) {
            f.insert("mode".into(), json!(crate::dialog_blend_ui::mode_key(mode)));
        }
        ui.add_space(6.0);
        ui.label(egui::RichText::new(tl!("Opacity:")).size(12.0).color(Tokens::get(ui.ctx()).text_dim));
        let mut opacity = f.get("opacity").and_then(Value::as_f64).unwrap_or(100.0) as f32;
        if crate::widgets::value_field(ui, &mut opacity, 0.0..=100.0, "%", 64.0).changed() {
            f.insert("opacity".into(), json!(opacity.round()));
        }
    });
}

#[cfg(test)]
mod tests;

/// The New Group body mirrors in a right-to-left UI: the rows read from the right, and every
/// field follows its caption.
#[cfg(test)]
mod rtl_tests {
    use super::*;
    use crate::direction::Dir;
    use egui_kittest::kittest::Queryable;

    /// The body drawn in `dir`, with the column it draws in.
    fn harness(dir: Dir) -> (egui_kittest::Harness<'static, (Map<String, Value>, egui::Rect)>, egui::Rect) {
        let _dir = crate::direction::direction_scope(dir);
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let f = fields(&app, NEW_GROUP);
        let mut h = egui_kittest::Harness::builder()
            .with_size(egui::vec2(460.0, 240.0))
            .with_context_setup(|ctx| PhotocraftApp::setup_context(ctx, crate::theme::ThemeKind::ALL[0]))
            .build_ui_state(
                |ui, s: &mut (Map<String, Value>, egui::Rect)| {
                    s.1 = ui.max_rect();
                    body(ui, &mut s.0);
                },
                (f, egui::Rect::ZERO),
            );
        h.run_steps(2);
        let rect = h.state().1;
        (h, rect)
    }

    /// The rows mirror: the caption comes before the field in a left-to-right UI and after it in
    /// a right-to-left one.
    #[test]
    fn the_rows_mirror_in_a_right_to_left_ui() {
        let (ltr, ltr_col) = harness(Dir::Ltr);
        let ltr_name = ltr.get_by_role(egui::accesskit::Role::TextInput).rect();
        let ltr_mode = ltr.get_by_label("Mode").rect();
        let ltr_opacity = ltr.get_by_label("Opacity:").rect();
        assert!(ltr_name.left() - ltr_col.left() > 50.0, "LTR: the caption comes before the field: {ltr_name:?} in {ltr_col:?}");
        assert!(ltr_mode.right() < ltr_opacity.left(), "LTR: the mode dropdown comes before the opacity field: {ltr_mode:?} {ltr_opacity:?}");

        let (rtl, rtl_col) = harness(Dir::Rtl);
        let rtl_name = rtl.get_by_role(egui::accesskit::Role::TextInput).rect();
        let rtl_mode = rtl.get_by_label("Mode").rect();
        let rtl_opacity = rtl.get_by_label("Opacity:").rect();
        assert!(rtl_col.right() - rtl_name.right() > 50.0, "RTL: the caption comes after the field: {rtl_name:?} in {rtl_col:?}");
        assert!(rtl_mode.left() > rtl_opacity.right(), "RTL: the row mirrors: {rtl_mode:?} {rtl_opacity:?}");
    }
}
