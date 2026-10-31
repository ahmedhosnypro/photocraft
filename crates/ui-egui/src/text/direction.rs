//! Right-to-left layout (Arabic, Hebrew, Persian, Urdu).
//!
//! The direction follows the UI language ([`crate::i18n::Lang::dir`]). In a right-to-left
//! language the UI mirrors, as in right-to-left macOS and Windows apps: the menu bar starts at
//! the right, the toolbox docks on the right, the dock and its rail on the left, and the title
//! and status bars read from the right. Dialogs, floating windows, docked panel interiors,
//! menus, popups and document tabs mirror too; the canvas itself stays left-to-right
//! (Photoshop-like), as does numeric text inside mirrored rows.
//!
//! Two mechanisms work together:
//! - the egui fork's `Style::right_to_left` flag ([`crate::theme::set_direction`], called from
//!   [`crate::theme::apply`] and on language changes from `i18n::sync_context`) mirrors every
//!   egui-owned widget: submenus and popups open reading-end first, vertical scrollbars sit on
//!   the left, collapsing-header triangles point left, checkbox/radio icons swap sides, and
//!   window title bars put the close button on the left; and
//! - these helpers mirror what egui does not: row and column order for explicit layouts, side
//!   panels, and hand-painted geometry. A nested `ui.horizontal` under a [`body_layout`] or
//!   [`row`] parent follows the row's direction on its own.
//!
//! Mirror each surface exactly once: the flag, a [`body_layout`] scope and an explicit
//! [`row`]/[`trailing`] pin each flip a row — never combine two on the same row. Pointer
//! semantics (scrub drags, slider value→x mapping) stay physical.
//!
//! `PHOTOCRAFT_UI_DIRECTION=rtl` (or `ltr`) forces a direction whatever the language, so the
//! mirrored layout can be checked before a right-to-left catalog is installed. It is read once
//! per process; the web build has no environment and always follows the language.

use std::cell::Cell;
use std::sync::OnceLock;

use egui::{Align, Layout};

pub use crate::i18n::Dir;

thread_local! {
    // Like the drawing language, per thread: test harnesses run side by side.
    static FORCED: Cell<Option<Dir>> = const { Cell::new(None) };
}

/// The direction the UI is drawn in on this thread.
pub fn current() -> Dir {
    FORCED.get().or_else(env_override).unwrap_or_else(|| crate::i18n::current().dir())
}

/// Is the UI drawn right-to-left?
pub fn is_rtl() -> bool {
    current() == Dir::Rtl
}

/// Draw in `dir` whatever the language until the guard drops on this thread (tests).
#[must_use]
pub fn direction_scope(dir: Dir) -> impl Drop {
    struct Restore {
        previous: Option<Dir>,
        _thread: std::marker::PhantomData<std::rc::Rc<()>>,
    }
    impl Drop for Restore {
        fn drop(&mut self) {
            FORCED.set(self.previous);
        }
    }
    let restore = Restore { previous: FORCED.get(), _thread: std::marker::PhantomData };
    FORCED.set(Some(dir));
    restore
}

fn env_override() -> Option<Dir> {
    static ENV: OnceLock<Option<Dir>> = OnceLock::new();
    *ENV.get_or_init(|| parse_dir(&std::env::var("PHOTOCRAFT_UI_DIRECTION").ok()?))
}

/// `rtl` / `ltr`, any case; anything else (including empty) follows the language.
fn parse_dir(value: &str) -> Option<Dir> {
    match value.trim().to_ascii_lowercase().as_str() {
        "rtl" => Some(Dir::Rtl),
        "ltr" => Some(Dir::Ltr),
        _ => None,
    }
}

/// A side panel on the reading-start side: left, or right in a right-to-left UI (the toolbox).
pub fn start_panel(id: impl egui::AsIdSalt) -> egui::Panel {
    if is_rtl() { egui::Panel::right(id) } else { egui::Panel::left(id) }
}

/// A side panel on the reading-end side: right, or left in a right-to-left UI (the dock).
pub fn end_panel(id: impl egui::AsIdSalt) -> egui::Panel {
    if is_rtl() { egui::Panel::left(id) } else { egui::Panel::right(id) }
}

/// A vertically centred row in reading order.
pub fn row() -> Layout {
    if is_rtl() { Layout::right_to_left(Align::Center) } else { Layout::left_to_right(Align::Center) }
}

/// A vertically centred row that starts from the reading end: for groups pinned to the far end
/// of a bar, such as the title bar's controls or the status bar's Fit button.
pub fn trailing() -> Layout {
    if is_rtl() { Layout::left_to_right(Align::Center) } else { Layout::right_to_left(Align::Center) }
}

/// [`egui::Ui::horizontal_centered`] in reading order: a row filling the space left, with its
/// items centred vertically, starting from the right in a right-to-left UI.
pub fn horizontal_centered<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let size = ui.available_size_before_wrap();
    ui.allocate_ui_with_layout(size, row(), add)
}

/// The popup alignment for a menu opening below its button: under its reading-start edge.
pub fn below() -> egui::RectAlign {
    if is_rtl() { egui::RectAlign::BOTTOM_END } else { egui::RectAlign::BOTTOM_START }
}

/// The body layout for dialogs and window bodies: in a right-to-left UI blocks right-align and
/// their nested `ui.horizontal` rows mirror; in a left-to-right UI it is egui's default.
///
/// egui mirrors a nested row only when the parent layout runs right-to-left, or is vertical with
/// `Align::Max` cross alignment; a `top_down(Align::Min)` body keeps every nested row
/// left-to-right even with the direction flag set, so mirrored bodies scope themselves with this.
pub fn body_layout() -> Layout {
    if is_rtl() { Layout::top_down(Align::Max) } else { Layout::top_down(Align::Min) }
}

/// A row hugging the reading end with top-aligned items: the footer shape for window and dialog
/// bodies ([`trailing`] centres its items).
pub fn trailing_min() -> Layout {
    if is_rtl() { Layout::left_to_right(Align::Min) } else { Layout::right_to_left(Align::Min) }
}

/// A full-width row in reading order for bodies that stay `top_down(Align::Min)`:
/// [`horizontal_centered`] without the vertical centring. Rows inside a [`body_layout`] parent
/// already mirror; opt in here only where that scope does not reach.
pub fn dialog_row<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let size = ui.available_size_before_wrap();
    ui.allocate_ui_with_layout(size, row_min(), add)
}

/// [`row`] with top-aligned items, for rows inside a vertical body.
fn row_min() -> Layout {
    if is_rtl() { Layout::right_to_left(Align::Min) } else { Layout::left_to_right(Align::Min) }
}

/// The horizontal keys meaning "next on screen" and "previous on screen":
/// `(ArrowRight, ArrowLeft)` left-to-right, swapped in a right-to-left UI. Menu-bar navigation
/// uses one tuple so the open-menu and bar-focus key machines cannot diverge.
pub fn horizontal_nav_keys() -> (egui::Key, egui::Key) {
    if is_rtl() { (egui::Key::ArrowLeft, egui::Key::ArrowRight) } else { (egui::Key::ArrowRight, egui::Key::ArrowLeft) }
}

/// Wrap a floating window's (or dialog's) content: in a right-to-left UI the body's blocks
/// right-align and its nested rows mirror; in a left-to-right UI the content draws unchanged.
/// Call once at the top of each `egui::Window` body closure.
pub fn window_body<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    if is_rtl() {
        ui.with_layout(body_layout(), add)
    } else {
        let inner = add(ui);
        egui::InnerResponse::new(inner, ui.response())
    }
}

/// A label inside a mirrored body ([`body_layout`]/[`window_body`]): in a right-to-left UI the
/// paragraph base direction is pinned right-to-left and the galley aligns to the reading-start
/// (right) edge, so mixed Arabic/Latin lines keep their phrase order instead of scrambling and
/// hug the right margin. In a left-to-right UI this is egui's plain label, unchanged.
pub fn body_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) -> egui::Response {
    let text = text.into();
    if !is_rtl() {
        return ui.label(text);
    }
    let mut job = std::sync::Arc::unwrap_or_clone(text.into_layout_job(ui.style(), egui::FontSelection::Default, ui.text_valign()));
    job.direction = egui::epaint::text::TextDirection::RightToLeft;
    job.halign = Align::Max;
    ui.label(egui::WidgetText::LayoutJob(std::sync::Arc::new(job)))
}

/// Run a block inside a left-to-right scope even in a right-to-left UI. `egui::Grid` places its
/// first cell at the raw cursor, which is `x = -inf` at the start of a right-to-left row, so a
/// grid must never sit directly in an RTL row; wrap it in this and mirror the cell order with
/// [`grid_cells`].
pub fn ltr<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    ui.with_layout(Layout::left_to_right(Align::Min), add)
}

/// [`ltr`] for a block in a vertical flow: keeps the top-down direction (and so the parent's
/// cursor advance) but keeps nested rows left-to-right, which a mirrored body
/// (`top_down(Align::Max)`) would not. Wrap direction-blind widgets such as `egui::Grid` in this.
pub fn ltr_vertical<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    ui.with_layout(Layout::top_down(Align::Min), add)
}

/// Two `egui::Grid` cells in reading order — label then editor left-to-right, editor then label
/// right-to-left. `egui::Grid` ignores the direction flag, so the column order is emitted
/// manually. Call once per row; end the row with `ui.end_row()` as usual.
pub fn grid_cells<L, E>(ui: &mut egui::Ui, label: L, editor: E)
where
    L: FnOnce(&mut egui::Ui),
    E: FnOnce(&mut egui::Ui),
{
    if is_rtl() {
        editor(ui);
        label(ui);
    } else {
        label(ui);
        editor(ui);
    }
}

/// One settings row without a `Grid`: the label occupies `label_w` at the reading start and the
/// editor follows toward the reading end — the mirrored shape of a two-column settings grid.
/// `label_w = 0.0` lets a bare checkbox hug the reading start.
pub fn settings_row<L, E>(ui: &mut egui::Ui, label_w: f32, label: L, editor: E)
where
    L: FnOnce(&mut egui::Ui),
    E: FnOnce(&mut egui::Ui),
{
    let width = ui.available_width();
    ui.allocate_ui_with_layout(egui::vec2(width, 0.0), row(), |ui| {
        ui.allocate_ui_with_layout(egui::vec2(label_w, 0.0), row(), |ui| label(ui));
        editor(ui);
        ui.add_space(7.0);
    });
}

#[cfg(test)]
mod tests;
