use crate::app::TabInner;
use crate::asset::ProxyMeta;
use crate::context::EditorContext;
use crate::global::SelectedEntity;
use crate::style::Style;
use bevy::ecs::schedule::{Schedule, SystemStage};
use bevy::prelude::{Children, Entity, Name, Parent, Visibility, Without};
use bevy::prelude::{Local, Query, Res, ResMut};
use bevy::utils::HashMap;
use egui::style::Margin;
use egui::widgets::TextEdit;
use egui::*;
use std::borrow::Cow;

#[derive(Default)]
pub struct Hierarchy {
    search: String,
    cache: HashMap<u32, usize>,
}

impl TabInner for Hierarchy {
    fn ui(&mut self, ui: &mut Ui, style: &Style, mut ctx: EditorContext) {
        self.cache.clear();
        self.cache.extend(
            ctx.scene
                .entities
                .iter()
                .enumerate()
                .map(|(index, entity)| (entity.entity, index)),
        );

        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            style.theme(ui);

            ui.horizontal(|ui| {
                let frame = Frame::none().margin(Margin::symmetric(3.0, 3.0));
                frame.fill(style.tab_base).show(ui, |ui| {
                    let text = TextEdit::singleline(&mut self.search);
                    ui.add(text.desired_width(f32::INFINITY));
                });
            });

            style.for_scrollbar(ui);

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.id_source("hierarchy scroll").show(ui, |ui| {
                style.scrollarea(ui);
                for index in 0..ctx.scene.entities.len() {
                    if ctx.get(index).unwrap().without::<Parent>() {
                        item_widget(0, index, ui, style, &self.cache, &mut ctx);
                    }
                }
            });
        });
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    open: bool,
}

impl State {
    fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data().get_temp(id)
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data().insert_temp(id, self);
    }

    pub fn toggle(&mut self, ui: &Ui) {
        self.open = !self.open;
        ui.ctx().request_repaint();
    }
}

fn item_widget(
    level: usize,
    entity: usize,
    ui: &mut Ui,

    style: &Style,
    cache: &HashMap<u32, usize>,
    ctx: &mut EditorContext,
) {
    let id = Id::new((entity, "#hierarchy_item"));
    let mut state = State::load(ui.ctx(), id).unwrap_or(State { open: true });
    let is_open = state.open;

    let fill_color = style.panel;
    let active_color = style.selection;
    let tri_color = style.input_text;
    let text_color = style.input_text;

    let full_width = ui.available_width();
    let full_size = vec2(full_width, 20.0);
    let (rect, response_bg) = ui.allocate_exact_size(full_size, Sense::hover());

    let bg_color = if response_bg.hovered() || ctx.selected_index(None) == Some(entity) {
        active_color
    } else {
        fill_color
    };

    ui.painter().rect_filled(response_bg.rect, 0.0, bg_color);

    let offset = vec2(16.0 * level as f32, 0.0);

    let tri_icon_pos = rect.left_center() + vec2(8.0, 0.0) + offset;
    let custom_icon_pos = rect.left_center() + vec2(23.0, 0.0) + offset;
    let label_pos = rect.left_center() + vec2(36.0, 0.0) + offset;
    let hide_icon_pos = rect.right_center() - vec2(16.0, 0.0);

    let cursor = CursorIcon::PointingHand;
    let sense = Sense::click();

    let mut editor = ctx.get(entity).unwrap();

    if let Some(icon) = editor.struct_field_mut::<ProxyMeta, u32>("icon") {
        let rect = Rect::from_center_size(custom_icon_pos, vec2(16.0, 20.0));
        let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);

        ui.painter().text(
            custom_icon_pos,
            Align2::CENTER_CENTER,
            char::from_u32(*icon).unwrap(),
            FontId::proportional(16.0),
            text_color,
        );
    }

    if let Some(name) = editor
        .struct_field_mut::<ProxyMeta, Cow<'static, str>>("name")
        .map(Cow::to_mut)
    {
        ui.painter().text(
            label_pos,
            Align2::LEFT_CENTER,
            name.clone(),
            FontId::proportional(14.0),
            text_color,
        );
    }

    if let Some(is_visible) = editor.struct_field_mut::<ProxyMeta, bool>("is_visible") {
        let rect = Rect::from_center_size(hide_icon_pos, vec2(16.0, 20.0));
        let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);
        if response.clicked() {
            *is_visible = !*is_visible;
        }
        ui.painter().text(
            hide_icon_pos,
            Align2::CENTER_CENTER,
            if *is_visible {
                crate::blender::HIDE_OFF
            } else {
                crate::blender::HIDE_ON
            },
            FontId::proportional(16.0),
            text_color,
        );
    }

    let has_children = editor
        .children()
        .map_or(false, |children| !children.is_empty());

    if has_children {
        let rect = Rect::from_center_size(tri_icon_pos, vec2(16.0, 20.0));
        let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);
        if response.clicked() {
            state.toggle(ui);
        }
        ui.painter().text(
            tri_icon_pos,
            Align2::CENTER_CENTER,
            if is_open {
                crate::blender::DISCLOSURE_TRI_DOWN
            } else {
                crate::blender::DISCLOSURE_TRI_RIGHT
            },
            FontId::proportional(16.0),
            tri_color,
        );
    }

    if let Some(children) = editor.children() {
        let children: Vec<u32> = children
            .iter()
            .map(|e| e.downcast_ref::<Entity>().unwrap().id())
            .collect();

        if is_open {
            for entity in children {
                let entity = cache[&entity];
                item_widget(level + 1, entity, ui, style, cache, ctx);
            }
        }
    }

    if response_bg.interact(egui::Sense::click()).clicked() {
        ctx.select(entity);
    }

    state.store(ui.ctx(), id);
}
