use bevy::asset::HandleId;
use bevy::prelude::*;
use bevy::reflect::*;
use egui::{DragValue, Ui, WidgetText};
use std::borrow::Cow;

const WRAP_WIDTH: f32 = 235.0;
pub const PERCENT: f32 = 0.35;

pub fn reflect(ui: &mut Ui, reflect: &mut dyn Reflect) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
        let builder = Builder {
            level: 0,
            wrapping: ui.available_width() > WRAP_WIDTH,
        };
        builder.field_reflect(ui, reflect);
    });
}

#[derive(Clone, Copy)]
struct Builder {
    level: usize,
    wrapping: bool,
}

impl Builder {
    fn next_level(self) -> Self {
        Self {
            level: self.level + 1,
            ..self
        }
    }

    fn field_reflect(&self, ui: &mut Ui, reflect: &mut dyn Reflect) {
        match reflect.reflect_mut() {
            ReflectMut::Struct(reflect) => self.field_struct(ui, reflect),
            ReflectMut::TupleStruct(reflect) => self.field_tuple_struct(ui, reflect),
            ReflectMut::Tuple(reflect) => self.field_tuple(ui, reflect),
            ReflectMut::List(reflect) => self.field_list(ui, reflect),
            ReflectMut::Map(reflect) => self.field_map(ui, reflect),
            ReflectMut::Value(reflect) => {
                macro_rules! drag {
                    ($($ty:ident),+) => {
                        $(
                            if let Some(value) = reflect.downcast_mut::<$ty>() {
                                ui.columns(1, |ui| {
                                    ui[0].add(DragValue::new(value).speed(0.1));
                                });
                                return;
                            }
                        )+
                    };
                }

                drag!(u8, u16, u32, u64, usize);
                drag!(i8, i16, i32, i64, isize);
                drag!(f32, f64);

                if let Some(value) = reflect.downcast_mut::<bool>() {
                    ui.columns(1, |ui| {
                        ui[0].checkbox(value, "");
                    });
                    return;
                }

                if let Some(value) = reflect.downcast_mut::<Cow<'static, str>>() {
                    ui.columns(1, |ui| {
                        ui[0].text_edit_singleline(value.to_mut());
                    });
                    return;
                }
                if let Some(value) = reflect.downcast_mut::<String>() {
                    ui.columns(1, |ui| {
                        ui[0].text_edit_singleline(value);
                    });
                    return;
                }

                if let Some(value) = reflect.downcast_mut::<Vec2>() {
                    let num = if self.wrapping { 2 } else { 1 };
                    ui.columns(num, |ui| {
                        ui[0.min(num - 1)].add(DragValue::new(&mut value.x).speed(0.1));
                        ui[1.min(num - 1)].add(DragValue::new(&mut value.y).speed(0.1));
                    });
                    return;
                }
                if let Some(value) = reflect.downcast_mut::<Vec3>() {
                    let num = if self.wrapping { 3 } else { 1 };
                    ui.columns(num, |ui| {
                        ui[0.min(num - 1)].add(DragValue::new(&mut value.x).speed(0.1));
                        ui[1.min(num - 1)].add(DragValue::new(&mut value.y).speed(0.1));
                        ui[2.min(num - 1)].add(DragValue::new(&mut value.z).speed(0.1));
                    });
                    return;
                }
                if let Some(value) = reflect.downcast_mut::<Vec4>() {
                    let num = if self.wrapping { 4 } else { 1 };
                    ui.columns(num, |ui| {
                        ui[0.min(num - 1)].add(DragValue::new(&mut value.x).speed(0.1));
                        ui[1.min(num - 1)].add(DragValue::new(&mut value.y).speed(0.1));
                        ui[2.min(num - 1)].add(DragValue::new(&mut value.z).speed(0.1));
                        ui[3.min(num - 1)].add(DragValue::new(&mut value.w).speed(0.1));
                    });
                    return;
                }
                if let Some(value) = reflect.downcast_mut::<Quat>() {
                    let euler = EulerRot::YXZ;
                    let (mut y, mut x, mut z) = value.to_euler(euler);

                    let num = if self.wrapping { 3 } else { 1 };
                    ui.columns(num, |ui| {
                        ui[0.min(num - 1)].drag_angle(&mut y);
                        ui[1.min(num - 1)].drag_angle(&mut x);
                        ui[2.min(num - 1)].drag_angle(&mut z);
                    });

                    *value = Quat::from_euler(euler, y, x, z);

                    return;
                }

                if let Some(value) = reflect.downcast_mut::<Color>() {
                    if let Color::Rgba {
                        red,
                        green,
                        blue,
                        alpha,
                    } = value
                    {
                        let mut rgba =
                            egui::Rgba::from_rgba_premultiplied(*red, *green, *blue, *alpha);
                        let alpha_mode = egui::widgets::color_picker::Alpha::Opaque;
                        egui::widgets::color_picker::color_edit_button_rgba(
                            ui, &mut rgba, alpha_mode,
                        );
                        *red = rgba.r();
                        *green = rgba.g();
                        *blue = rgba.b();
                        *alpha = rgba.a();
                    }

                    return;
                }

                if let Some(value) = reflect.downcast_mut::<HandleId>() {
                    ui.label(format!("{:?}", value));
                    return;
                }

                if let Some(value) = reflect.downcast_mut::<Entity>() {
                    ui.label(format!("{:?}", value));
                    return;
                }

                ui.label(reflect.type_name());
            }
        }
    }

    fn field_struct(&self, ui: &mut Ui, reflect: &mut (dyn Struct + 'static)) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            for index in 0..reflect.field_len() {
                if let Some(label) = reflect.name_at(index).map(WidgetText::from) {
                    let field = reflect.field_at_mut(index).unwrap();
                    self.field_pair(ui, label, field);
                } else {
                    let field = reflect.field_at_mut(index).unwrap();
                    self.next_level().field_reflect(ui, field);
                }
            }
        });
    }

    fn field_tuple_struct(&self, ui: &mut Ui, reflect: &mut (dyn TupleStruct + 'static)) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            for index in 0..reflect.field_len() {
                let field = reflect.field_mut(index).unwrap();
                self.field_pair(ui, index.to_string(), field);
            }
        });
    }

    fn field_tuple(&self, ui: &mut Ui, reflect: &mut (dyn Tuple + 'static)) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            for index in 0..reflect.field_len() {
                let field = reflect.field_mut(index).unwrap();
                self.field_pair(ui, index.to_string(), field);
            }
        });
    }

    fn field_list(&self, ui: &mut Ui, reflect: &mut (dyn List + 'static)) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            for index in 0..reflect.len() {
                let field = reflect.get_mut(index).unwrap();
                self.field_pair(ui, index.to_string(), field);
            }
        });
    }

    fn field_map(&self, ui: &mut Ui, reflect: &mut (dyn Map + 'static)) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            for index in 0..reflect.len() {
                let (key, _) = reflect.get_at(index).unwrap();
                let key = unsafe { crate::util::fuck_ref(key) };
                let field = reflect.get_mut(key).unwrap();
                self.next_level().field_reflect(ui, field);
            }
        });
    }

    fn field_pair(&self, ui: &mut Ui, label: impl Into<WidgetText>, reflect: &mut dyn Reflect) {
        let width = ui.available_width();
        let height = 18.0;
        let pad = 16.0;

        let run_label = |ui: &mut egui::Ui| {
            let indent = pad * self.level as f32;
            let _ = ui.allocate_space(egui::vec2(indent, height));
            let (id, space) = ui.allocate_space(egui::vec2(width * PERCENT - indent, height));
            let layout = egui::Layout::left_to_right();
            let mut ui = ui.child_ui_with_id_source(space, layout, id);
            ui.label(label);
        };

        if self.wrapping && matches!(reflect.reflect_ref(), ReflectRef::Value(_)) {
            ui.spacing_mut().interact_size.y = height;
            ui.horizontal(|ui| {
                run_label(ui);
                self.field_reflect(ui, reflect);
            });
        } else {
            ui.horizontal(|ui| {
                run_label(ui);
            });
            self.next_level().field_reflect(ui, reflect);
        }
    }
}
