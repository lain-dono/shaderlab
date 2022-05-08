use crate::style::Style;
use bevy::prelude::*;
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
            .layout_no_wrap(self.label, font_id, self.style.tab_text);

        let offset = egui::vec2(8.0, 0.0);
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
            ui.painter().rect_filled(tab, rounding, self.style.tab_base);
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

/*
pub struct App {
    pub ctx: egui::Context,
    pub state: egui_winit::State,
    pub rpass: RenderPass,

    pub tree: SplitTree,
    pub style: Style,
    pub global: Global,

    pub drag_start: Option<egui::Pos2>,
}

impl App {
    pub fn new(
        device: &wgpu::Device,
        window: &Window,
        output_format: wgpu::TextureFormat,
        sample_count: u32,
        root: TreeNode,
    ) -> Self {
        let limits = device.limits();
        let max_texture_side = limits.max_texture_dimension_2d as usize;

        Self {
            state: egui_winit::State::new(max_texture_side, window),
            rpass: RenderPass::new(device, output_format, sample_count),
            ctx: {
                let context = egui::Context::default();
                context.set_fonts(crate::global::fonts_with_blender());
                context
            },

            tree: SplitTree::new(root),
            style: Style::default(),
            global: Global::default(),

            drag_start: None,
        }
    }

    pub fn on_event(&mut self, event: WindowEvent) {
        self.state.on_event(&self.ctx, &event);
        if let Some(event) = event.to_static() {
            let parent_scale = self.ctx.pixels_per_point();

            for node in self.tree.iter_mut() {
                if let TreeNode::Leaf {
                    tabs,
                    active,
                    viewport,
                    ..
                } = node
                {
                    tabs[*active]
                        .inner
                        .on_event(&event, *viewport, parent_scale);
                }
            }
        }
    }

    pub fn run(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: Target,
    ) -> Result<bool, BackendError> {
        let new_input = self.state.take_egui_input(window);
        self.ctx.begin_frame(new_input);

        {
            let id = egui::Id::new("#_SHADERLAB_#");
            let layer_id = egui::LayerId::background();
            let rect = self.ctx.available_rect();

            let mut ui = egui::Ui::new(self.ctx.clone(), layer_id, id, rect, rect);
            self.draw(&mut ui, rect);
        }

        {
            fn linear_from_srgb(r: f64, g: f64, b: f64) -> [f64; 3] {
                let cutoff = [r < 0.04045, g < 0.04045, b < 0.04045];
                let lower = [r / 12.92, g / 12.92, b / 12.92];
                let higher = [
                    ((r + 0.055) / 1.055).powf(2.4),
                    ((g + 0.055) / 1.055).powf(2.4),
                    ((b + 0.055) / 1.055).powf(2.4),
                ];
                [
                    if cutoff[0] { lower[0] } else { higher[0] },
                    if cutoff[1] { lower[1] } else { higher[1] },
                    if cutoff[2] { lower[2] } else { higher[2] },
                ]
            }

            let [r, g, b, _] = self.style.app_bg.to_srgba_unmultiplied();
            let [r, g, b] = [r as f64, g as f64, b as f64];
            let [r, g, b] = [r / 255.0, g / 255.0, b / 255.0];
            let [r, g, b] = linear_from_srgb(r, g, b);
            let a = 1.0;
            let clear = wgpu::Color { r, g, b, a };

            let attachment = target.attach(true, clear);

            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui main render pass"),
                color_attachments: &[attachment],
                depth_stencil_attachment: None,
            });
        }

        let egui::FullOutput {
            shapes,
            needs_repaint,
            textures_delta,
            platform_output,
        } = self.ctx.end_frame();

        let scale = self.ctx.pixels_per_point();

        for node in self.tree.iter_mut() {
            if let TreeNode::Leaf {
                tabs,
                active,
                viewport,
                ..
            } = node
            {
                let ctx = RenderContext {
                    window,
                    device,
                    queue,
                    encoder,
                    attachment: target.attach(true, None),
                    viewport: self::panel::rect_scale(*viewport, scale),
                };
                tabs[*active].inner.render(ctx);
            }
        }

        self.state
            .handle_platform_output(window, &self.ctx, platform_output);

        let size = window.inner_size();
        let screen_descriptor = ScreenDescriptor {
            width: size.width,
            height: size.height,
            scale: self.ctx.pixels_per_point(),
        };

        let paint_jobs = self.ctx.tessellate(shapes);

        self.rpass.add_textures(device, queue, &textures_delta)?;
        self.rpass.remove_textures(textures_delta)?;
        self.rpass
            .update_buffers(device, queue, &paint_jobs, &screen_descriptor);
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui main render pass"),
                color_attachments: &[target.attach(true, None)],
                depth_stencil_attachment: None,
            });

            rpass.push_debug_group("egui_pass");

            self.rpass.execute(
                &mut rpass,
                &paint_jobs,
                &screen_descriptor,
                Rect::EVERYTHING,
            )?;

            rpass.pop_debug_group();
        }

        Ok(needs_repaint)
    }
}
*/

fn nested_menus(ui: &mut egui::Ui) {
    if ui.button("New...").clicked() {
        ui.close_menu();
    }
    if ui.button("Open...").clicked() {
        ui.close_menu();
    }
}

#[allow(clippy::only_used_in_recursion)]
pub fn ui_root(
    mut drag_start: Local<Option<egui::Pos2>>,
    mut context: ResMut<crate::shell::EguiContext>,
    mut tree: ResMut<SplitTree>,
    mut scene: ResMut<Scene>,
    style: Res<Style>,
) {
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

            ui.menu_button("File", nested_menus);
            ui.menu_button("Edit", nested_menus);
            ui.menu_button("Assets", nested_menus);
            ui.menu_button("Objects", nested_menus);
            ui.menu_button("Components", nested_menus);
            ui.menu_button("Window", nested_menus);
        });

        response.rect
    };

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
            rect.intersect(Rect::everything_left_of(rect.min.x + separator)),
            rect.intersect(Rect::everything_right_of(rect.max.x - separator)),
        ];
        for rect in corners {
            ui.painter().rect_filled(rect, 0.0, style.app_bg);
        }
        rect.shrink(separator)
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

                    let mut ui = ui.child_ui(rect, Default::default());
                    tab.inner.ui(&mut ui, &style, &mut scene.world);

                    scene.world.insert_resource(ui);
                    tab.schedule.run(&mut scene.world);
                    scene.world.remove_resource::<egui::Ui>();
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
                    tree.split(dst, TreeNode::leaf(tab), target);
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
