use crate::app::{Style, TabInner};
use crate::global::{EditorEntityState, Global, Icon};
use bevy::prelude::{Children, Entity, Name, Parent, Query, Visibility, Without, World};
use egui::style::Margin;
use egui::widgets::TextEdit;
use egui::*;

#[derive(Default)]
pub struct Hierarchy {
    search: String,
}

impl TabInner for Hierarchy {
    fn ui(&mut self, ui: &mut Ui, style: &Style, global: &mut Global) {
        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_gray(0x28));

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.horizontal(|ui| {
                let frame = Frame::none().margin(Margin::symmetric(3.0, 3.0));
                frame.fill(style.tab_base).show(ui, |ui| {
                    let text = TextEdit::singleline(&mut self.search);
                    ui.add(text.desired_width(f32::INFINITY));
                });
            });

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.id_source("hierarchy scroll").show(ui, |ui| {
                let mut query = global.world.query_filtered::<Entity, Without<Parent>>();
                for entity in query.iter(crate::fuck_ref(&global.world)) {
                    item_widget(0, ui, style, global, entity);
                }
            });
        });
    }
}

fn item_widget(level: usize, ui: &mut Ui, style: &Style, global: &mut Global, entity: Entity) {
    let fill_color = Color32::from_gray(0x2d);
    let active_color = Color32::from_gray(0x3e);
    let tri_color = Color32::from_gray(0x5e);
    let text_color = Color32::from_gray(0xa8);

    let full_width = ui.available_width();
    let full_size = vec2(full_width, 20.0);
    let (rect, response_bg) = ui.allocate_exact_size(full_size, Sense::hover());

    if response_bg.hovered() || global.selected == Some(entity) {
        let shape = Shape::rect_filled(response_bg.rect, 0.0, active_color);
        ui.painter().add(shape);
    } else {
        let shape = Shape::rect_filled(response_bg.rect, 0.0, fill_color);
        ui.painter().add(shape);
    }

    let icon_size = 16.0;
    let offset = vec2(14.0 * level as f32, 0.0);

    let tri_icon_pos = rect.left_center() + vec2(icon_size * 0.5, 0.0) + offset;
    let custom_icon_pos = rect.left_center() + vec2(icon_size * 1.375, 0.0) + offset;
    let label_pos = rect.left_center() + vec2(icon_size * 2.0, 0.0) + offset;
    let hide_icon_pos = rect.right_center() - vec2(icon_size, 0.0);

    let cursor = CursorIcon::PointingHand;
    let sense = Sense::click();

    let mut icon_query = global.world.query::<&mut Icon>();
    if let Ok(icon) = icon_query.get(&global.world, entity) {
        let rect = Rect::from_center_size(custom_icon_pos, vec2(icon_size, 20.0));
        let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);

        ui.painter().text(
            custom_icon_pos,
            Align2::CENTER_CENTER,
            icon.icon,
            FontId::proportional(16.0),
            text_color,
        );
    }

    let mut name_query = global.world.query::<&mut Name>();
    if let Ok(name) = name_query.get(&global.world, entity) {
        ui.painter().text(
            label_pos,
            Align2::LEFT_CENTER,
            name.as_ref(),
            FontId::proportional(14.0),
            text_color,
        );
    }

    let mut visibility_query = global.world.query::<&mut Visibility>();
    if let Ok(mut visibility) = visibility_query.get_mut(&mut global.world, entity) {
        let rect = Rect::from_center_size(hide_icon_pos, vec2(icon_size, 20.0));
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

    let mut children_query = global.world.query::<&mut Children>();

    let has_children = children_query
        .get(&global.world, entity)
        .map_or(false, |children| !children.is_empty());

    let mut ed_query = global.world.query::<&mut EditorEntityState>();
    let is_open = if let Ok(mut ed) = ed_query.get_mut(&mut global.world, entity) {
        if has_children {
            let rect = Rect::from_center_size(tri_icon_pos, vec2(icon_size, 20.0));
            let response = ui.allocate_rect(rect, sense).on_hover_cursor(cursor);
            if response.clicked() {
                ed.is_open = !ed.is_open;
            }
            ui.painter().text(
                tri_icon_pos,
                Align2::CENTER_CENTER,
                if ed.is_open {
                    crate::blender::DISCLOSURE_TRI_DOWN
                } else {
                    crate::blender::DISCLOSURE_TRI_RIGHT
                },
                FontId::proportional(18.0),
                tri_color,
            );
        }

        ed.is_open
    } else {
        true
    };

    if response_bg.interact(egui::Sense::click()).clicked() {
        global.selected = Some(entity);
    }

    if let Ok(children) = children_query.get(crate::fuck_ref(&global.world), entity) {
        if is_open {
            for &entity in children.iter() {
                item_widget(level + 1, ui, style, global, entity);
            }
        }
    }
}
