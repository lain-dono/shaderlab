use super::{ComponentEditor, Proxy, ReflectComponentEditor, ReflectProxy};
use crate::ui::Style;
use bevy::prelude::*;
use bevy::reflect::{DynamicStruct, FromReflect};
use egui::*;
use std::borrow::Cow;

#[derive(Component, Reflect, FromReflect)]
#[reflect(Component, Proxy, ComponentEditor)]
pub struct ProxyMeta {
    pub icon: u32,
    pub name: Cow<'static, str>,
    pub is_visible: bool,
}

impl ProxyMeta {
    pub fn new(icon: char, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            icon: icon as u32,
            name: name.into(),
            is_visible: true,
        }
    }
}

impl Default for ProxyMeta {
    fn default() -> Self {
        Self::new('?', "Entity")
    }
}

impl Proxy for ProxyMeta {
    fn insert(self, world: &mut World, entity: Entity) {
        let mut v = ComputedVisibility::not_visible();
        v.set_visible_in_view();

        world.entity_mut(entity).insert_bundle((
            Name::new(self.name),
            Visibility {
                is_visible: self.is_visible,
            },
            v,
        ));
    }
}

impl ComponentEditor for ProxyMeta {
    fn ui(ui: &mut Ui, style: &Style, component: &mut dyn Reflect) {
        let data = component.downcast_mut::<DynamicStruct>().unwrap();

        ui.horizontal(|ui| {
            let frame = Frame::none().inner_margin(style::Margin::symmetric(2.0, 3.0));
            frame.fill(style.tab_base).show(ui, |ui| {
                /*
                if false {
                    let icon = if self.lock.is_some() {
                        crate::blender::LOCKED
                    } else {
                        crate::blender::UNLOCKED
                    };
                    let widget = Button::new(icon.to_string()).frame(false);
                    if ui.add(widget).clicked() {
                        if self.lock.is_some() {
                            self.lock.take();
                        } else {
                            self.lock = Some(entity.index);
                        }
                    }
                    ui.add_space(4.0);
                }
                */

                let icon_field = data.get_field_mut::<u32>("icon").unwrap();

                let icon_char = char::from_u32(*icon_field).unwrap();
                let icon = text::LayoutJob::simple_singleline(
                    icon_char.into(),
                    FontId::proportional(16.0),
                    style.input_text,
                );

                let InnerResponse { inner, response } = ui.menu_button(icon, |ui| {
                    style.for_scrollbar(ui);
                    let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
                    scroll.id_source("inspector icons").show(ui, |ui| {
                        style.set_theme_visuals(ui);
                        style.scrollarea(ui);
                        ui.set_width(300.0);
                        ui.horizontal_wrapped(|ui| {
                            for c in 0xE900..=0xEB99 {
                                let c = char::from_u32(c).unwrap();
                                if ui.button(String::from(c)).clicked() {
                                    ui.close_menu();
                                    return Some(c);
                                }
                            }

                            None
                        })
                    })
                });
                response.on_hover_cursor(egui::CursorIcon::PointingHand);

                if let Some(icon) = inner.and_then(|r| r.inner.inner) {
                    *icon_field = icon as u32;
                }

                ui.add_space(4.0);

                let name = data.get_field_mut::<Cow<'static, str>>("name").unwrap();

                let text = TextEdit::singleline(name.to_mut());
                ui.add(text.desired_width(f32::INFINITY));
            });
        });
    }
}
