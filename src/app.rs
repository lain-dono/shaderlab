use crate::asset::ReflectScene;
use crate::context::EditorContext;
use crate::style::Style;
use crate::util::anymap::AnyMap;
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::window::WindowId;
use egui::Rect;

//pub mod backend;
//pub mod panel;
pub mod tabs;

pub use self::tabs::{NodeIndex, RenderContext, Split, SplitTree, Tab, TabInner, TreeNode};

struct TabWidget<'a> {
    label: String,
    active: bool,
    style: &'a Style,
}

impl<'a> egui::Widget for TabWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let px = ui.ctx().pixels_per_point().recip();
        let rounding = self.style.tab_rounding;

        let font_id = egui::FontId::proportional(14.0);
        let galley = ui
            .painter()
            .layout_no_wrap(self.label, font_id, self.style.input_text);

        let offset = egui::vec2(10.0, 0.0);
        let text_size = galley.size();

        let mut desired_size = text_size + offset * 2.0;
        desired_size.y = 24.0;

        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::hover());
        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

        if self.active {
            let mut tab = rect;

            tab.min.x -= px;
            tab.max.x += px;
            ui.painter()
                .rect_filled(tab, rounding, self.style.tab_outline);

            tab.min.x += px;
            tab.max.x -= px;
            ui.painter().rect_filled(tab, rounding, self.style.panel);
        }

        let pos = egui::Align2::LEFT_TOP
            .anchor_rect(rect.shrink2(egui::vec2(8.0, 5.0)))
            .min;

        ui.painter().galley(pos, galley);

        response
    }
}

struct HoverData {
    rect: Rect,
    tabs: Option<Rect>,
    dst: NodeIndex,
    pointer: egui::Pos2,
}

impl HoverData {
    fn resolve(&self) -> (Option<Split>, Rect) {
        if let Some(tabs) = self.tabs {
            return (None, tabs);
        }

        let (rect, pointer) = (self.rect, self.pointer);

        let center = rect.center();
        let pts = [
            center.distance(pointer),
            rect.left_center().distance(pointer),
            rect.right_center().distance(pointer),
            rect.center_top().distance(pointer),
            rect.center_bottom().distance(pointer),
        ];

        let position = pts
            .into_iter()
            .enumerate()
            .min_by(|(_, lhs), (_, rhs)| crate::util::total_cmp(lhs, rhs))
            .map(|(idx, _)| idx)
            .unwrap();

        let (target, other) = match position {
            0 => (None, Rect::EVERYTHING),
            1 => (Some(Split::Left), Rect::everything_left_of(center.x)),
            2 => (Some(Split::Right), Rect::everything_right_of(center.x)),
            3 => (Some(Split::Above), Rect::everything_above(center.y)),
            4 => (Some(Split::Below), Rect::everything_below(center.y)),
            _ => unreachable!(),
        };

        (target, rect.intersect(other))
    }
}

#[allow(clippy::only_used_in_recursion)]
pub fn ui_root(
    mut drag_start: Local<Option<egui::Pos2>>,
    mut context: ResMut<crate::shell::EguiContext>,
    mut tree: ResMut<SplitTree>,
    scene: Res<Handle<ReflectScene>>,
    mut state: ResMut<AnyMap>,
    mut scenes: ResMut<Assets<ReflectScene>>,
    style: Res<Style>,
    type_registry: Res<TypeRegistry>,
    mut assets: ResMut<AssetServer>,
) {
    let scene = scenes.get_mut(scene.clone()).unwrap();

    let (rect, mut ui) = {
        let [ctx] = context.ctx_mut([WindowId::primary()]);

        {
            let mut style = egui::Style::clone(&ctx.style());

            style.visuals.widgets.noninteractive.rounding = egui::Rounding::none();
            style.visuals.widgets.inactive.rounding = egui::Rounding::none();
            style.visuals.widgets.hovered.rounding = egui::Rounding::none();
            style.visuals.widgets.active.rounding = egui::Rounding::none();
            style.visuals.widgets.open.rounding = egui::Rounding::none();
            style.visuals.window_rounding = egui::Rounding::none();

            ctx.set_style(style);
        }

        let rect = ctx.available_rect();
        let ui = egui::Ui::new(
            ctx.clone(),
            egui::LayerId::background(),
            egui::Id::new("#_SHADERLAB_#"),
            rect,
            rect,
        );

        (rect, ui)
    };

    let tabbar_bg_idx = ui.painter().add(egui::Shape::Noop);

    let topbar = {
        let egui::InnerResponse { response, .. } = egui::menu::bar(&mut ui, |ui| {
            ui.spacing_mut().button_padding.x += 2.0;
            ui.spacing_mut().button_padding.y += 4.0;

            let space = ui.spacing().button_padding.x;
            ui.add_space(space * 2.0);

            //style.theme(ui);
            {
                let mut visuals = ui.visuals().clone();
                visuals.widgets.noninteractive.bg_stroke.width = 0.0;
                visuals.widgets.inactive.bg_stroke.width = 0.0;
                visuals.widgets.hovered.bg_stroke.width = 0.0;
                visuals.widgets.active.bg_stroke.width = 0.0;
                visuals.widgets.open.bg_stroke.width = 0.0;

                visuals.popup_shadow = egui::epaint::Shadow::default();

                ui.ctx().set_visuals(visuals);
            }

            fn nested_menus(ui: &mut egui::Ui) {
                if ui.button("New...").clicked() {
                    ui.close_menu();
                }
                if ui.button("Open...").clicked() {
                    ui.close_menu();
                }

                ui.menu_button("Next", nested_menus);
            }

            ui.menu_button("File", nested_menus);
            ui.menu_button("Edit", nested_menus);
            ui.menu_button("Assets", nested_menus);
            ui.menu_button("Objects", nested_menus);
            ui.menu_button("Components", nested_menus);
            ui.menu_button("Window", nested_menus);
        });

        response.rect
    };

    style.theme(&mut ui);

    ui.painter().set(
        tabbar_bg_idx,
        egui::Shape::rect_filled(topbar, 0.0, style.app_bg),
    );

    if tree.is_empty() || tree[NodeIndex::root()].is_none() {
        ui.painter().rect_filled(rect, 0.0, style.app_bg);
        // TODO: splash screen here?
        return;
    }

    let rect = {
        let rect = rect.intersect(Rect::everything_below(topbar.height()));
        let separator = style.separator_size;
        let corners = [
            rect.intersect(Rect::everything_above(rect.min.y + separator)),
            rect.intersect(Rect::everything_below(rect.max.y - separator)),
            rect.intersect(Rect::everything_left_of(rect.min.x + separator + 2.0)),
            rect.intersect(Rect::everything_right_of(rect.max.x - separator - 2.0)),
        ];
        for rect in corners {
            ui.painter().rect_filled(rect, 0.0, style.app_bg);
        }
        rect.shrink2(egui::vec2(separator + 2.0, separator))
    };

    tree[NodeIndex::root()].set_rect(rect);

    let mut drag_data = None;
    let mut hover_data = None;

    let pixels_per_point = ui.ctx().pixels_per_point();
    let px = pixels_per_point.recip();

    for tree_index in 0..tree.len() {
        let tree_index = NodeIndex(tree_index);
        match &mut tree[tree_index] {
            TreeNode::None => (),

            TreeNode::Horizontal { fraction, rect } => {
                let rect = crate::util::expand_to_pixel(*rect, pixels_per_point);
                ui.set_clip_rect(rect);

                let (left, separator, right) = style.hsplit(&mut ui, fraction, rect);
                ui.painter().rect_filled(separator, 0.0, style.app_bg);

                tree[tree_index.left()].set_rect(left);
                tree[tree_index.right()].set_rect(right);
            }

            TreeNode::Vertical { fraction, rect } => {
                let rect = crate::util::expand_to_pixel(*rect, pixels_per_point);
                ui.set_clip_rect(rect);

                let (bottom, separator, top) = style.vsplit(&mut ui, fraction, rect);
                ui.painter().rect_filled(separator, 0.0, style.app_bg);

                tree[tree_index.left()].set_rect(bottom);
                tree[tree_index.right()].set_rect(top);
            }

            TreeNode::Leaf {
                rect,
                tabs,
                active,
                viewport,
            } => {
                let rect = *rect;
                ui.set_clip_rect(rect);

                let height_topbar = 24.0;

                let bottom_y = rect.min.y + height_topbar;
                let tabbar = rect.intersect(Rect::everything_above(bottom_y));

                let full_response = ui.allocate_rect(rect, egui::Sense::hover());
                let tabs_response = ui.allocate_rect(tabbar, egui::Sense::hover());

                // tabs
                {
                    ui.painter().rect_filled(tabbar, 0.0, style.app_bg);
                    ui.painter()
                        .rect_filled(tabbar, style.tab_rounding, style.tab_bar);

                    let a = egui::pos2(tabbar.min.x, tabbar.max.y - px);
                    let b = egui::pos2(tabbar.max.x, tabbar.max.y - px);
                    ui.painter().line_segment([a, b], (px, style.tab_outline));

                    let mut ui = ui.child_ui(tabbar, Default::default());
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    ui.horizontal(|ui| {
                        for (tab_index, tab) in tabs.iter().enumerate() {
                            let widget = TabWidget {
                                label: tab.to_string(),
                                active: *active == tab_index,
                                style: &style,
                            };

                            let id = egui::Id::new((tree_index, tab_index, "tab"));
                            let is_being_dragged = ui.memory().is_being_dragged(id);

                            if is_being_dragged {
                                let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
                                let response =
                                    ui.with_layer_id(layer_id, |ui| ui.add(widget)).response;

                                let sense = egui::Sense::click_and_drag();
                                let response = ui
                                    .interact(response.rect, id, sense)
                                    .on_hover_cursor(egui::CursorIcon::Grabbing);

                                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                                    let center = response.rect.center();
                                    let start = drag_start.unwrap_or(center);

                                    let delta = pointer_pos - start;
                                    if delta.x.abs() > 30.0 || delta.y.abs() > 6.0 {
                                        ui.ctx().translate_layer(layer_id, delta);

                                        drag_data = Some((tree_index, tab_index));
                                    }
                                }

                                if response.clicked() {
                                    *active = tab_index;
                                }
                            } else {
                                let response = ui.scope(|ui| ui.add(widget)).response;
                                let sense = egui::Sense::click_and_drag();
                                let response = ui.interact(response.rect, id, sense);
                                if response.drag_started() {
                                    *drag_start = response.hover_pos();
                                }
                            }
                        }
                    });
                }

                // tab body
                if let Some(tab) = tabs.get_mut(*active) {
                    let top_y = rect.min.y + height_topbar;
                    let rect = rect.intersect(Rect::everything_below(top_y));
                    let rect = crate::util::expand_to_pixel(rect, pixels_per_point);

                    *viewport = rect;

                    let ctx = EditorContext {
                        scene,
                        state: &mut state,
                        types: &type_registry,
                        assets: &mut assets,
                    };

                    let mut ui = ui.child_ui(rect, Default::default());
                    tab.inner.ui(&mut ui, &style, ctx);
                }

                let is_being_dragged = ui.memory().is_anything_being_dragged();
                if is_being_dragged && full_response.hovered() {
                    hover_data = ui.input().pointer.hover_pos().map(|pointer| HoverData {
                        rect,
                        dst: tree_index,
                        tabs: tabs_response.hovered().then(|| tabs_response.rect),
                        pointer,
                    });
                }
            }
        }
    }

    if let (Some((src, tab_index)), Some(hover)) = (drag_data, hover_data) {
        let dst = hover.dst;

        if tree[src].is_leaf() && tree[dst].is_leaf() {
            let (target, helper) = hover.resolve();

            let id = egui::Id::new("helper");
            let layer_id = egui::LayerId::new(egui::Order::Foreground, id);
            let painter = ui.ctx().layer_painter(layer_id);
            painter.rect_filled(helper, 0.0, style.selection);

            if ui.input().pointer.any_released() {
                if let TreeNode::Leaf { active, .. } = &mut tree[src] {
                    if *active >= tab_index {
                        *active = active.saturating_sub(1);
                    }
                }

                let tab = tree[src].remove_tab(tab_index).unwrap();

                if let Some(target) = target {
                    tree.split(dst, target, 0.5, TreeNode::leaf(tab));
                } else {
                    tree[dst].append_tab(tab);
                }

                tree.remove_empty_leaf();
                for node in tree.iter_mut() {
                    if let TreeNode::Leaf { tabs, active, .. } = node {
                        if *active >= tabs.len() {
                            *active = 0;
                        }
                    }
                }
            }
        }
    }
}
