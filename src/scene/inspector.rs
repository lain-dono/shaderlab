use super::component::{reflect_component_editor, ComponentEditor, ReflectComponentEditor};
use super::context::EditorContext;
use super::ReflectScene;
use crate::ui::{EditorTab, Style};
use crate::util::anymap::AnyMap;
use bevy::ecs::system::lifetimeless::{SRes, SResMut};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::{FromType, Reflect, TypeRegistry};
use egui::style::Margin;
use egui::*;

#[derive(Default, Component)]
pub struct Inspector {
    lock: Option<usize>,
}

impl EditorTab for Inspector {
    type Param = (
        SRes<Style>,
        SRes<Handle<ReflectScene>>,
        SResMut<AnyMap>,
        SResMut<Assets<ReflectScene>>,
        SRes<TypeRegistry>,
        SResMut<AssetServer>,
    );

    fn ui<'w>(
        &mut self,
        ui: &mut egui::Ui,
        _entity: Entity,
        (style, scene, state, scenes, types, assets): &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let scene = scenes.get_mut(scene).unwrap();
        let mut ctx = EditorContext {
            scene,
            state,
            types,
            assets,
        };

        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        let entity = match ctx.find_selected(self.lock) {
            Some(entity) => entity,
            None => {
                Frame::none()
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.vertical_centered_justified(|ui| ui.label("Select something..."));
                    });
                return;
            }
        };

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            style.set_theme_visuals(ui);
            style.for_scrollbar(ui);

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.show(ui, |ui| {
                style.scrollarea(ui);

                let frame = Frame::none();
                frame.fill(style.panel).show(ui, |ui| {
                    for component in entity.entity.components.iter_mut().map(AsMut::as_mut) {
                        let type_name = component.type_name();

                        {
                            use bevy::prelude::*;
                            add_custom_editor_if::<Parent>(entity.types, type_name);
                            add_custom_editor_if::<Children>(entity.types, type_name);
                        }

                        let registry = entity.types.read();
                        let registration = match registry.get_with_name(type_name) {
                            Some(registration) => registration,
                            None => continue,
                        };

                        if let Some(editor) = registration.data::<ReflectComponentEditor>() {
                            if editor.skip() {
                                continue;
                            }
                            editor.ui(ui, style, component);
                        } else {
                            let name = registration.short_name();
                            reflect_component_editor(ui, style, component, ' ', name);
                        }

                        let width = ui.available_width();
                        let (_, separator) = ui.allocate_space(vec2(width, 1.0));
                        ui.painter().rect_filled(separator, 0.0, style.separator);
                    }
                });
            });
        });
    }
}

fn add_custom_editor_if<T: ComponentEditor + Reflect>(types: &TypeRegistry, type_name: &str) {
    if type_name == std::any::type_name::<T>() {
        let mut registry = types.write();
        if let Some(registration) = registry.get_with_name_mut(type_name) {
            let data: ReflectComponentEditor = FromType::<T>::from_type();
            registration.insert(data);
        }
    }
}
