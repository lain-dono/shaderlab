use crate::app::TabInner;
use crate::context::EditorContext;
use crate::style::Style;
use bevy::reflect::{DynamicStruct, FromType, Reflect, Struct};
use bevy_reflect::TypeRegistry;
use egui::style::Margin;
use egui::text::LayoutJob;
use egui::widgets::TextEdit;
use egui::*;
use std::borrow::Cow;

pub mod field;

#[derive(Default)]
pub struct Inspector {
    lock: Option<usize>,
}

impl TabInner for Inspector {
    fn ui(&mut self, ui: &mut Ui, style: &Style, mut ctx: EditorContext) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, style.panel);

        let entity = match ctx.find_selected(self.lock) {
            Some(entity) => entity,
            None => {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Select something...");
                });
                return;
            }
        };

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            style.theme(ui);
            style.for_scrollbar(ui);

            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
            scroll.show(ui, |ui| {
                style.scrollarea(ui);

                let mut type_registry = entity.type_registry.write();

                let frame = Frame::none();
                frame.fill(style.panel).show(ui, |ui| {
                    for component in entity.entity.components.iter_mut().map(AsMut::as_mut) {
                        let type_name = component.type_name();

                        {
                            use bevy::prelude::*;
                            add_custom_editor_if::<Parent>(&mut type_registry, type_name);
                            add_custom_editor_if::<PreviousParent>(&mut type_registry, type_name);
                            add_custom_editor_if::<Children>(&mut type_registry, type_name);
                        }

                        let registration = match type_registry.get_with_name(type_name) {
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

fn add_custom_editor_if<T: ComponentEditor + Reflect>(
    type_registry: &mut TypeRegistry,
    type_name: &str,
) {
    if type_name == std::any::type_name::<T>() {
        if let Some(registration) = type_registry.get_with_name_mut(type_name) {
            let data: ReflectComponentEditor = FromType::<T>::from_type();
            registration.insert(data);
        }
    }
}

#[derive(Clone, Debug, Default)]
struct State {
    open: bool,
}

impl State {
    fn load(ctx: &Context, id: Id) -> Self {
        ctx.data().get_temp(id).unwrap_or(Self { open: true })
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data().insert_temp(id, self);
    }

    pub fn toggle(&mut self, ui: &Ui) {
        self.open = !self.open;
        ui.ctx().request_repaint();
    }
}

fn reflect_component_editor(
    ui: &mut egui::Ui,
    style: &Style,
    reflect: &mut dyn Reflect,
    icon: char,
    name: &str,
) {
    let id = Id::new((reflect.type_name(), "#component_header"));
    let mut state = State::load(ui.ctx(), id);

    if component_header(Some(&mut state), ui, style, icon, name) {
        let margin = Margin {
            left: 6.0,
            right: 2.0,
            top: 4.0,
            bottom: 6.0,
        };
        Frame::none().margin(margin).show(ui, |ui| {
            self::field::reflect(ui, reflect);
        });
    }

    state.store(ui.ctx(), id);
}

fn component_header(
    state: Option<&mut State>,
    ui: &mut egui::Ui,
    style: &Style,
    icon: char,
    name: &str,
) -> bool {
    let tri_color = style.input_text;
    let text_color = style.input_text;

    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    let tri_pos = rect.left_center() + vec2(8.0, 0.0);
    let icon_pos = rect.left_center() + vec2(24.0, 0.0);
    let label_pos = rect.left_center() + vec2(38.0, 0.0);
    let dots_pos = rect.right_center() - vec2(12.0, 0.0);

    ui.painter().text(
        icon_pos,
        Align2::CENTER_CENTER,
        icon.to_string(),
        FontId::proportional(16.0),
        text_color,
    );

    ui.painter().text(
        label_pos,
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(14.0),
        text_color,
    );

    ui.painter().text(
        dots_pos,
        Align2::CENTER_CENTER,
        crate::blender::THREE_DOTS,
        FontId::proportional(16.0),
        text_color,
    );

    if let Some(state) = state {
        if response.clicked() {
            state.toggle(ui);
        }
        ui.painter().text(
            tri_pos,
            Align2::CENTER_CENTER,
            if state.open {
                crate::blender::DISCLOSURE_TRI_DOWN
            } else {
                crate::blender::DISCLOSURE_TRI_RIGHT
            },
            FontId::proportional(16.0),
            tri_color,
        );
        state.open
    } else {
        false
    }
}

pub trait ComponentEditor {
    fn desc() -> (char, Cow<'static, str>) {
        (' ', "".into())
    }

    fn skip() -> bool {
        false
    }

    fn ui(ui: &mut Ui, style: &Style, reflect: &mut dyn Reflect) {
        let (icon, name) = Self::desc();
        reflect_component_editor(ui, style, reflect, icon, &name);
    }
}

#[derive(Clone)]
pub struct ReflectComponentEditor {
    skip: fn() -> bool,
    ui: fn(&mut Ui, style: &Style, &mut dyn Reflect),
}

impl ReflectComponentEditor {
    fn skip(&self) -> bool {
        (self.skip)()
    }

    fn ui(&self, ui: &mut Ui, style: &Style, reflect: &mut dyn Reflect) {
        (self.ui)(ui, style, reflect);
    }
}

impl<T: ComponentEditor + Reflect> FromType<T> for ReflectComponentEditor {
    fn from_type() -> Self {
        Self {
            skip: T::skip,
            ui: T::ui,
        }
    }
}

impl ComponentEditor for bevy::prelude::Parent {
    fn skip() -> bool {
        true
    }
}

impl ComponentEditor for bevy::prelude::PreviousParent {
    fn skip() -> bool {
        true
    }
}

impl ComponentEditor for bevy::prelude::Children {
    fn skip() -> bool {
        true
    }
}

impl ComponentEditor for crate::asset::ProxyTransform {
    fn desc() -> (char, Cow<'static, str>) {
        (crate::blender::ORIENTATION_LOCAL, "Transform".into())
    }
}

impl<T: bevy::asset::Asset> ComponentEditor for crate::asset::ProxyHandle<T> {
    fn desc() -> (char, Cow<'static, str>) {
        let type_name = std::any::type_name::<T>();
        if type_name == std::any::type_name::<bevy::prelude::Mesh>() {
            (crate::blender::MESH_DATA, "Mesh".into())
        } else if type_name == std::any::type_name::<bevy::prelude::StandardMaterial>() {
            (crate::blender::MATERIAL_DATA, "StandardMaterial".into())
        } else {
            (' ', format!("Handle<{}>", type_name).into())
        }
    }
}

impl ComponentEditor for crate::asset::ProxyMeta {
    fn ui(ui: &mut Ui, style: &Style, component: &mut dyn Reflect) {
        let data = component.downcast_mut::<DynamicStruct>().unwrap();

        ui.horizontal(|ui| {
            let frame = Frame::none().margin(Margin::symmetric(2.0, 3.0));
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

                let icon_field = unwrap_field_mut::<u32>(data, "icon");

                let icon_char = char::from_u32(*icon_field).unwrap();
                let icon = LayoutJob::simple_singleline(
                    icon_char.into(),
                    FontId::proportional(16.0),
                    style.input_text,
                );

                let InnerResponse { inner, response } = ui.menu_button(icon, |ui| {
                    style.for_scrollbar(ui);
                    let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
                    scroll.id_source("inspector icons").show(ui, |ui| {
                        style.theme(ui);
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

                let name = unwrap_field_mut::<Cow<'static, str>>(data, "name");

                let text = TextEdit::singleline(name.to_mut());
                ui.add(text.desired_width(f32::INFINITY));
            });
        });
    }
}

pub fn unwrap_field_ref<'a, T: Reflect>(data: &'a DynamicStruct, name: &str) -> &'a T {
    data.field(name).unwrap().downcast_ref::<T>().unwrap()
}

pub fn unwrap_field_mut<'a, T: Reflect>(data: &'a mut DynamicStruct, name: &str) -> &'a mut T {
    data.field_mut(name).unwrap().downcast_mut::<T>().unwrap()
}
