use crate::app::TabInner;
use crate::global::{Icon, SelectedEntity};
use crate::style::Style;
use bevy::ecs::schedule::{Schedule, SystemStage};
use bevy::prelude::{Children, Entity, Name, Parent, Visibility, Without};
use bevy::prelude::{Local, Query, Res, ResMut};
use egui::style::Margin;
use egui::widgets::TextEdit;
use egui::*;

#[derive(Default)]
pub struct Hierarchy;

impl Hierarchy {
    pub fn schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule.add_stage("main", SystemStage::single(hierarchy));
        schedule
    }
}

impl TabInner for Hierarchy {}

type All<'a> = (
    Option<&'a Children>,
    Option<&'a mut Icon>,
    Option<&'a mut Name>,
    Option<&'a mut Visibility>,
);

fn hierarchy(
    mut search: Local<String>,
    mut ui: ResMut<egui::Ui>,
    style: Res<Style>,
    mut selected: ResMut<SelectedEntity>,
    root: Query<Entity, Without<Parent>>,
    mut query: Query<All>,
) {
    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_gray(0x28));

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        ui.horizontal(|ui| {
            let frame = Frame::none().margin(Margin::symmetric(3.0, 3.0));
            frame.fill(style.tab_base).show(ui, |ui| {
                let text = TextEdit::singleline(&mut *search);
                ui.add(text.desired_width(f32::INFINITY));
            });
        });

        let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
        scroll.id_source("hierarchy scroll").show(ui, |ui| {
            for entity in root.iter() {
                item_widget(0, entity, &mut selected, ui, &mut query);
            }
        });
    });
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
    entity: Entity,
    selected: &mut ResMut<SelectedEntity>,
    ui: &mut Ui,
    query: &mut Query<All>,
) {
    let id = Id::new((entity, "#hierarchy_item"));
    let mut state = State::load(ui.ctx(), id).unwrap_or(State { open: true });
    let is_open = state.open;

    let fill_color = Color32::from_gray(0x2d);
    let active_color = Color32::from_gray(0x3e);
    let tri_color = Color32::from_gray(0x5e);
    let text_color = Color32::from_gray(0xa8);

    let full_width = ui.available_width();
    let full_size = vec2(full_width, 20.0);
    let (rect, response_bg) = ui.allocate_exact_size(full_size, Sense::hover());

    if response_bg.hovered() || selected.0 == Some(entity) {
        let shape = Shape::rect_filled(response_bg.rect, 0.0, active_color);
        ui.painter().add(shape);
    } else {
        let shape = Shape::rect_filled(response_bg.rect, 0.0, fill_color);
        ui.painter().add(shape);
    }

    let offset = vec2(14.0 * level as f32, 0.0);

    let tri_icon_pos = rect.left_center() + vec2(8.0, 0.0) + offset;
    let custom_icon_pos = rect.left_center() + vec2(22.0, 0.0) + offset;
    let label_pos = rect.left_center() + vec2(32.0, 0.0) + offset;
    let hide_icon_pos = rect.right_center() - vec2(16.0, 0.0);

    let cursor = CursorIcon::PointingHand;
    let sense = Sense::click();

    let (children, icon, name, visibility) = query.get_mut(entity).unwrap();

    if let Some(icon) = icon {
        let rect = Rect::from_center_size(custom_icon_pos, vec2(16.0, 20.0));
        let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);

        ui.painter().text(
            custom_icon_pos,
            Align2::CENTER_CENTER,
            icon.get(),
            FontId::proportional(16.0),
            text_color,
        );
    }

    if let Some(name) = name {
        ui.painter().text(
            label_pos,
            Align2::LEFT_CENTER,
            name.as_ref(),
            FontId::proportional(14.0),
            text_color,
        );
    }

    if let Some(mut visibility) = visibility {
        let rect = Rect::from_center_size(hide_icon_pos, vec2(16.0, 20.0));
        let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);
        if response.clicked() {
            visibility.is_visible = !visibility.is_visible;
        }
        ui.painter().text(
            hide_icon_pos,
            Align2::CENTER_CENTER,
            if visibility.is_visible {
                crate::blender::HIDE_OFF
            } else {
                crate::blender::HIDE_ON
            },
            FontId::proportional(16.0),
            text_color,
        );
    }

    let has_children = children.map_or(false, |children| !children.is_empty());

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
            FontId::proportional(18.0),
            tri_color,
        );
    }

    if response_bg.interact(egui::Sense::click()).clicked() {
        selected.0 = Some(entity);
    }

    if let Some(children) = children {
        if is_open {
            for &entity in children.clone().iter() {
                item_widget(level + 1, entity, selected, ui, query);
            }
        }
    }

    state.store(ui.ctx(), id);
}
