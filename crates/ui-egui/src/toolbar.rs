//! Optimised PDF: the user's own tools in the toolbar (the mode bar, after Save, Save As and
//! Print). Any command a tool category offers can be pinned there: right-click the toolbar (or use
//! its pin button) for the categories, hover one for all of its tools, and click a tool to pin or
//! unpin it. A pinned button runs its command; dragging it moves it along the row; right-clicking
//! it removes it. The order is kept in the settings (`toolbar_tools`), and the control channel sets
//! it with `toolbar-tools`.

use egui::{Rect, Sense, Stroke, Vec2};
use pdfcraft_engine::catalog::{Availability, TOOL_GROUPS};
use pdfcraft_engine::commands;

use crate::PdfCraftApp;
use crate::theme::Tokens;
use crate::widgets::icons;

/// The most tools the toolbar holds: beyond this a row of icons stops being quicker than a menu.
pub const MAX_TOOLBAR_TOOLS: usize = 24;

/// Size of a pinned button, as the other icon buttons in the bars.
const BUTTON: f32 = 32.0;

/// One tool category of the right-click menu: its label, icon and tools (command id, label), with
/// `None` between the category's sections.
pub struct Category {
    pub label: &'static str,
    pub icon: &'static str,
    pub tools: Vec<Option<(&'static str, String)>>,
}

/// The categories and their tools, as View ▸ Tools lists them: every ready tool of every ready
/// category, including each Add/Update/Remove variant, that names a command the app runs. A
/// command offered by several categories is listed under each.
pub fn categories() -> Vec<Category> {
    TOOL_GROUPS
        .iter()
        .filter(|g| g.availability == Availability::Ready)
        .filter_map(|group| {
            let mut tools = Vec::new();
            let mut seen: Vec<&str> = Vec::new();
            for section in group.sections {
                let before = tools.len();
                for tool in section.items.iter().filter(|t| t.availability == Availability::Ready) {
                    let actions: &[&'static str] = if tool.actions.is_empty() { std::slice::from_ref(&tool.command) } else { tool.actions };
                    for &id in actions {
                        let Some(spec) = commands::command(id) else { continue };
                        if seen.contains(&id) {
                            continue;
                        }
                        seen.push(id);
                        let label = if tool.actions.is_empty() { tl!(tool.label).to_string() } else { crate::i18n::menu_label(id, spec.label) };
                        tools.push(Some((id, label)));
                    }
                }
                if tools.len() > before {
                    tools.push(None);
                }
            }
            tools.pop();
            (!tools.is_empty()).then_some(Category { label: group.label, icon: group.icon, tools })
        })
        .collect()
}

/// `ids` cleaned up: only commands the app knows, each once, in their order, at most
/// [`MAX_TOOLBAR_TOOLS`].
pub fn normalize(ids: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    ids.retain(|id| commands::command(id).is_some() && seen.insert(id.clone()));
    ids.truncate(MAX_TOOLBAR_TOOLS);
}

/// Where the button at `from` lands when dropped with the pointer at `x`: past every other button
/// whose centre is left of the pointer.
fn drop_index(slots: &[Rect], from: usize, x: f32) -> usize {
    slots.iter().enumerate().filter(|(i, r)| *i != from && r.center().x < x).count()
}

impl PdfCraftApp {
    /// Whether command `id` is pinned to the toolbar.
    pub fn is_on_toolbar(&self, id: &str) -> bool {
        self.toolbar_tools.iter().any(|t| t == id)
    }

    /// Pin command `id` to the end of the toolbar. False when it isn't a command, is already
    /// there, or the toolbar is full.
    pub fn pin_to_toolbar(&mut self, id: &str) -> bool {
        if commands::command(id).is_none() || self.is_on_toolbar(id) || self.toolbar_tools.len() >= MAX_TOOLBAR_TOOLS {
            return false;
        }
        self.toolbar_tools.push(id.to_string());
        true
    }

    /// Take command `id` off the toolbar. False when it wasn't there.
    pub fn unpin_from_toolbar(&mut self, id: &str) -> bool {
        let before = self.toolbar_tools.len();
        self.toolbar_tools.retain(|t| t != id);
        self.toolbar_tools.len() != before
    }

    /// Move the toolbar's button at `from` so that it ends up at position `to` (both counted along
    /// the row). Positions past the end are clamped.
    pub fn move_toolbar_tool(&mut self, from: usize, to: usize) {
        if from >= self.toolbar_tools.len() {
            return;
        }
        let id = self.toolbar_tools.remove(from);
        let to = to.min(self.toolbar_tools.len());
        self.toolbar_tools.insert(to, id);
    }

    /// Replace the toolbar's tools with `ids`, a comma-separated list of command ids (the control
    /// channel's `toolbar-tools`). Unknown ids are an error, so a typo doesn't empty the toolbar.
    pub fn set_toolbar_tools(&mut self, ids: &str) -> Result<(), String> {
        let mut tools: Vec<String> = ids.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect();
        if let Some(bad) = tools.iter().find(|id| commands::command(id).is_none()) {
            return Err(format!("toolbar-tools: `{bad}` is not a command (see the command list)"));
        }
        if tools.len() > MAX_TOOLBAR_TOOLS {
            return Err(format!("toolbar-tools: at most {MAX_TOOLBAR_TOOLS} tools"));
        }
        normalize(&mut tools);
        self.toolbar_tools = tools;
        Ok(())
    }
}

/// The right-click menu: a submenu per category with all of its tools, each pinned or unpinned on
/// a click (a pinned one is shown selected), and a way to clear the toolbar.
pub(crate) fn pin_menu(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    ui.set_min_width(200.0);
    ui.label(egui::RichText::new(tl!("Add tools to the toolbar")).strong());
    ui.separator();
    let color = ui.visuals().text_color();
    for category in categories() {
        let button = egui::Button::image_and_text(icons::image(category.icon, 16.0, color), tl!(category.label));
        egui::containers::menu::SubMenuButton::from_button(button).ui(ui, |ui| {
            ui.set_min_width(220.0);
            for tool in &category.tools {
                let Some((id, label)) = tool else {
                    ui.separator();
                    continue;
                };
                let Some(spec) = commands::command(id) else { continue };
                let pinned = app.is_on_toolbar(id);
                let full = !pinned && app.toolbar_tools.len() >= MAX_TOOLBAR_TOOLS;
                let button = egui::Button::image_and_text(icons::image(spec.icon, 16.0, color), label.as_str()).selected(pinned);
                let resp = ui.add_enabled(!full, button);
                let resp = if pinned { resp.on_hover_text(tl!("On the toolbar: click to remove it")) } else { resp };
                if resp.clicked() {
                    if pinned {
                        app.unpin_from_toolbar(id);
                    } else {
                        app.pin_to_toolbar(id);
                    }
                }
            }
        });
    }
    if !app.toolbar_tools.is_empty() {
        ui.separator();
        if ui.button(tl!("Remove all tools from the toolbar")).clicked() {
            app.toolbar_tools.clear();
            ui.close();
        }
    }
}

/// The pinned buttons and the pin button after them, in the mode bar's row.
/// Width [`buttons`] takes: the pinned tools and the pin button, with egui's gaps between them.
pub(crate) fn width(app: &PdfCraftApp, ui: &egui::Ui) -> f32 {
    let n = app.toolbar_tools.iter().filter(|id| commands::command(id).is_some()).count() as f32 + 1.0;
    n * BUTTON + (n - 1.0) * ui.spacing().item_spacing.x
}

pub(crate) fn buttons(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    let mut slots = Vec::with_capacity(app.toolbar_tools.len());
    let mut run = None;
    let mut remove = None;
    let mut dragged = None;
    for (k, id) in app.toolbar_tools.clone().iter().enumerate() {
        let Some(spec) = commands::command(id) else { continue };
        let label = crate::i18n::menu_label(id, spec.label);
        let enabled = app.active.is_some() || matches!(spec.needs, commands::Needs::Nothing) || app.command_enabled(spec);
        let tip = crate::commands::command_tip(ui.ctx(), &format!("{label} ({{key}})"), id);
        let tip = if tip.ends_with(" ()") || tip.contains("({key})") { label.clone() } else { tip };
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(BUTTON), Sense::click_and_drag());
        let resp = crate::theme::hand(resp).on_hover_text(&tip);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &label));
        let tint = if enabled { t.icon } else { t.icon.gamma_multiply(0.4) };
        if resp.hovered() || resp.dragged() {
            ui.painter().rect_filled(rect, t.radius, ui.visuals().widgets.hovered.weak_bg_fill);
        }
        icons::paint(ui, rect, spec.icon, (BUTTON * 0.5).round(), tint);
        if enabled && resp.clicked() {
            run = Some(id.clone());
        }
        if resp.dragged() || resp.drag_stopped() {
            dragged = Some((k, resp.drag_stopped()));
        }
        resp.context_menu(|ui| {
            if ui.button(tl!("Remove from toolbar")).clicked() {
                remove = Some(id.clone());
                ui.close();
            }
            ui.separator();
            pin_menu(app, ui);
        });
        slots.push(rect);
    }
    // Moving a button along the row: a line shows where it lands.
    if let Some((from, released)) = dragged
        && let Some(pos) = ui.ctx().pointer_latest_pos()
    {
        let to = drop_index(&slots, from, pos.x);
        if released {
            app.move_toolbar_tool(from, to);
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            if to != from {
                let others: Vec<&Rect> = slots.iter().enumerate().filter(|(i, _)| *i != from).map(|(_, r)| r).collect();
                let x = match to.checked_sub(1) {
                    None => others.first().map(|r| r.left() - 1.0),
                    Some(before) => others.get(before).map(|r| r.right() + 1.0),
                };
                if let (Some(x), Some(first)) = (x, slots.first()) {
                    ui.painter().vline(x, first.y_range().shrink(4.0), Stroke::new(2.0, t.accent));
                }
            }
        }
    }
    let pin = icons::button(ui, "pin", BUTTON, false, tl!("Add tools to the toolbar"));
    egui::Popup::menu(&pin).show(|ui| pin_menu(app, ui));
    if let Some(id) = remove {
        app.unpin_from_toolbar(&id);
    }
    if let Some(id) = run {
        app.run_command(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_tool_is_a_command_and_rotate_pages_is_offered() {
        let all = categories();
        assert!(!all.is_empty());
        for c in &all {
            assert!(!c.tools.is_empty(), "{}", c.label);
            assert!(c.tools.first().is_some_and(Option::is_some) && c.tools.last().is_some_and(Option::is_some), "no stray separators: {}", c.label);
            for (id, _) in c.tools.iter().flatten() {
                assert!(commands::command(id).is_some(), "{id}");
            }
        }
        let organize = all.iter().find(|c| c.label == "Organize pages").expect("Organize pages");
        assert!(organize.tools.iter().flatten().any(|(id, _)| id.starts_with("page.rotate")), "Rotate pages is a sub-tool");
    }

    #[test]
    fn pinning_moving_and_removing_tools() {
        let mut app = PdfCraftApp::new();
        assert!(app.toolbar_tools.is_empty(), "empty until the user pins something");
        let ids: Vec<&str> = categories().iter().flat_map(|c| c.tools.iter().flatten().map(|(id, _)| *id)).take(3).collect();
        for id in &ids {
            assert!(app.pin_to_toolbar(id));
        }
        assert!(!app.pin_to_toolbar(ids[0]), "once each");
        assert!(!app.pin_to_toolbar("no.such.command"));
        assert_eq!(app.toolbar_tools, ids);
        app.move_toolbar_tool(0, 2);
        assert_eq!(app.toolbar_tools, [ids[1], ids[2], ids[0]]);
        app.move_toolbar_tool(2, 0);
        assert_eq!(app.toolbar_tools, ids);
        app.move_toolbar_tool(9, 0);
        assert_eq!(app.toolbar_tools, ids, "a position past the end changes nothing");
        assert!(app.unpin_from_toolbar(ids[1]));
        assert!(!app.unpin_from_toolbar(ids[1]));
        assert_eq!(app.toolbar_tools, [ids[0], ids[2]]);
    }

    #[test]
    fn the_toolbar_is_kept_in_the_settings_and_set_by_the_control_channel() {
        let mut app = PdfCraftApp::new();
        app.set_toolbar_tools("file.save, print.dialog").unwrap();
        assert!(app.set_toolbar_tools("file.save,no.such").is_err());
        assert_eq!(app.toolbar_tools, ["file.save", "print.dialog"], "a bad list changes nothing");
        app.set_option("toolbar-tools", "print.dialog,file.save").unwrap();
        let saved = app.persist();
        let mut restored = PdfCraftApp::new();
        restored.restore(&saved);
        assert_eq!(restored.toolbar_tools, ["print.dialog", "file.save"]);
        // Unknown or repeated ids in old settings are dropped, not kept.
        let mut v: serde_json::Value = serde_json::from_str(&saved).unwrap();
        v["toolbar_tools"] = serde_json::json!(["file.save", "gone.command", "file.save", 7]);
        let mut cleaned = PdfCraftApp::new();
        cleaned.restore(&v.to_string());
        assert_eq!(cleaned.toolbar_tools, ["file.save"]);
    }

    #[test]
    fn a_dropped_button_lands_past_the_centres_it_crossed() {
        let slots: Vec<Rect> = (0..4).map(|i| Rect::from_min_size(egui::pos2(i as f32 * 34.0, 0.0), Vec2::splat(32.0))).collect();
        assert_eq!(drop_index(&slots, 0, 70.0), 1, "past the second button's centre");
        assert_eq!(drop_index(&slots, 0, 500.0), 3, "to the end");
        assert_eq!(drop_index(&slots, 3, -10.0), 0, "to the front");
    }
}
