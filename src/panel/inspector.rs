use crate::app::TabInner;
use crate::global::{Icon, SelectedEntity};
use crate::style::Style;
use bevy::ecs::component::{Component, ComponentId, StorageType};
use bevy::ecs::schedule::{Schedule, SystemStage};
use bevy::ecs::world::EntityMut;
use bevy::prelude::{Entity, IntoExclusiveSystem, Name, World};
use bevy::utils::HashMap;
use egui::style::Margin;
use egui::text::LayoutJob;
use egui::widgets::TextEdit;
use egui::*;

#[derive(Default)]
pub struct Inspector {
    lock: Option<Entity>,
}

impl Inspector {
    pub fn schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule.add_stage("main", SystemStage::single(draw.exclusive_system()));
        schedule
    }
}

impl TabInner for Inspector {
    /*
    fn ui(&mut self, ui: &mut Ui, style: &Style, world: &mut World) {
        return;

        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_gray(0x28));

        let entity = self
            .lock
            .or_else(|| world.resource::<SelectedEntity>().0)
            .and_then(|entity| world.get_entity_mut(entity));

        if let Some(mut entity) = entity {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
                scroll.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let frame = Frame::none().margin(Margin::symmetric(3.0, 3.0));
                        frame.fill(style.tab_base).show(ui, |ui| {
                            {
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
                                        self.lock = Some(entity.id());
                                    }
                                }
                            }

                            ui.add_space(3.0);

                            if let Some(mut state) = entity.get_mut::<Icon>() {
                                let icon = LayoutJob::simple_singleline(
                                    state.icon.into(),
                                    FontId::proportional(20.0),
                                    style.tab_text,
                                );

                                let InnerResponse { inner, .. } = ui.menu_button(icon, |ui| {
                                    let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
                                    scroll.id_source("inspector icons").show(ui, |ui| {
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

                                if let Some(icon) = inner.and_then(|r| r.inner.inner) {
                                    state.icon = icon;
                                }

                                ui.add_space(3.0);
                            }

                            if let Some(mut name) = entity.get_mut::<Name>() {
                                name.mutate(|text| {
                                    let text = TextEdit::singleline(text);
                                    ui.add(text.desired_width(f32::INFINITY));
                                })
                            }
                        });
                    });

                    for i in 0..8 {
                        ui.collapsing(format!("Heading #{}", i), |ui| {
                            for _ in 0..8 {
                                ui.label("Contents");
                            }
                        });
                    }
                });
            });
        } else {
            ui.vertical_centered_justified(|ui| {
                ui.label("Select something...");
            });
        }
    }
    */
}

fn draw(world: &mut World) {
    let selected = world.resource::<SelectedEntity>().0;
    let style = world.resource::<Style>().clone();
    let mut ui = world.remove_resource::<egui::Ui>().unwrap();

    let rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_gray(0x28));

    let entity = selected.and_then(|entity| world.get_entity_mut(entity));

    let mut entity = if let Some(entity) = entity {
        entity
    } else {
        ui.vertical_centered_justified(|ui| {
            ui.label("Select something...");
        });
        return;
    };

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
        scroll.show(ui, |ui| {
            ui.horizontal(|ui| {
                let frame = Frame::none().margin(Margin::symmetric(3.0, 3.0));
                frame.fill(style.tab_base).show(ui, |ui| {
                    ui.add_space(3.0);

                    if let Some(mut state) = entity.get_mut::<Icon>() {
                        let icon = LayoutJob::simple_singleline(
                            state.get().into(),
                            FontId::proportional(20.0),
                            style.tab_text,
                        );

                        let InnerResponse { inner, .. } = ui.menu_button(icon, |ui| {
                            let scroll = ScrollArea::vertical().auto_shrink([false; 2]);
                            scroll.id_source("inspector icons").show(ui, |ui| {
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

                        if let Some(icon) = inner.and_then(|r| r.inner.inner) {
                            state.set(icon);
                        }

                        ui.add_space(3.0);
                    }

                    if let Some(mut name) = entity.get_mut::<Name>() {
                        name.mutate(|text| {
                            let text = TextEdit::singleline(text);
                            ui.add(text.desired_width(f32::INFINITY));
                        })
                    }
                });
            });

            let count = entity.world().components().len();

            let editors = EditorManager::sample(entity.world());

            for component_id in (0..count).map(ComponentId::new) {
                if entity.contains_id(component_id) {
                    if entity.world().components().get_info(component_id).is_none() {
                        continue;
                    }

                    /*
                    if !editors.has(component_id) {
                        continue;
                    }
                    */

                    component_collapse(ui, component_id, &mut entity, &editors);
                }
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

fn component_collapse(
    ui: &mut egui::Ui,
    component_id: ComponentId,
    entity: &mut EntityMut<'_>,
    editors: &EditorManager,
) {
    let id = Id::new((component_id, "#component_editor"));
    let info = entity.world().components().get_info(component_id).unwrap();

    let mut state = State::load(ui.ctx(), id).unwrap_or(State { open: true });
    let is_open = state.open;

    let top_line = Color32::from_gray(0x15);
    let header_fill = Color32::from_gray(0x32);
    let bot_line = Color32::from_gray(0x26);
    let body_bg = Color32::from_gray(0x2d);

    let tri_color = Color32::from_gray(0x57);
    let text_color = Color32::from_gray(0xa5);

    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 20.0), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    if response.clicked() {
        state.toggle(ui);
    }

    let px = ui.ctx().pixels_per_point().recip();
    ui.painter().rect_filled(rect, 0.0, header_fill);

    let top = [pos2(rect.min.x, rect.min.y), pos2(rect.max.x, rect.min.y)];
    ui.painter().line_segment(top, (px, top_line));

    let tri_pos = rect.left_center() + vec2(8.0, 0.0);
    let icon_pos = rect.left_center() + vec2(22.0, 0.0);
    let label_pos = rect.left_center() + vec2(34.0, 0.0);

    let dots_pos = rect.right_center() - vec2(12.0, 0.0);

    ui.painter().text(
        tri_pos,
        Align2::CENTER_CENTER,
        if is_open {
            crate::blender::DISCLOSURE_TRI_DOWN
        } else {
            crate::blender::DISCLOSURE_TRI_RIGHT
        },
        FontId::proportional(18.0),
        tri_color,
    );

    if let Some(icon) = editors.icon(component_id) {
        ui.painter().text(
            icon_pos,
            Align2::CENTER_CENTER,
            icon.to_string(),
            FontId::proportional(16.0),
            text_color,
        );
    }

    let name = editors.name(component_id).unwrap_or_else(|| info.name());

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

    if is_open {
        let top = [pos2(rect.min.x, rect.max.y), pos2(rect.max.x, rect.max.y)];
        ui.painter().line_segment(top, (px, bot_line));
        ui.add_space(px);
        let bg_idx = ui.painter().add(Shape::Noop);

        let InnerResponse { response, .. } = Frame::none().show(ui, |ui| {
            if editors.has(component_id) {
                unsafe { editors.call_ui(entity, component_id, ui) };
            } else {
                for _ in 0..3 {
                    ui.label("1234");
                }
            }
        });

        let bg_rect = Rect::from_min_size(
            response.rect.min,
            vec2(rect.width(), response.rect.height()),
        );

        ui.painter()
            .set(bg_idx, Shape::rect_filled(bg_rect, 0.0, body_bg));
    }

    state.store(ui.ctx(), id);
}

struct RawEditor {
    icon: char,
    name: String,
    ui: unsafe fn(&mut Ui, *mut u8),
}

#[derive(Default)]
struct EditorManager {
    editors: HashMap<ComponentId, RawEditor>,
}

impl EditorManager {
    fn sample(world: &World) -> Self {
        use bevy::prelude::*;

        let mut builder = Self::default();

        unsafe {
            builder.insert::<Transform, _>(
                world,
                crate::blender::ORIENTATION_LOCAL,
                "Transform",
                |ui, tx| {
                    ui_transform(
                        ui,
                        &mut tx.translation,
                        &mut tx.rotation,
                        &mut tx.scale,
                        "lt",
                    )
                },
            );

            builder.insert::<bevy::prelude::GlobalTransform, _>(
                world,
                crate::blender::ORIENTATION_GLOBAL,
                "GlobalTransform",
                |ui, tx| {
                    ui_transform(
                        ui,
                        &mut tx.translation,
                        &mut tx.rotation,
                        &mut tx.scale,
                        "gt",
                    )
                },
            );
        }

        builder
    }

    /// # Safety
    /// N/A
    unsafe fn insert<T: Component, S: Into<String>>(
        &mut self,
        world: &World,
        icon: char,
        name: S,
        ui: unsafe fn(ui: &mut Ui, data: &mut T),
    ) {
        let type_id = std::any::TypeId::of::<T>();
        let id = world.components().get_id(type_id).unwrap();
        let raw = RawEditor {
            icon,
            name: name.into(),
            ui: std::mem::transmute(ui),
        };
        self.editors.insert(id, raw);
    }

    fn icon(&self, id: ComponentId) -> Option<char> {
        self.editors.get(&id).map(|e| e.icon)
    }

    fn name(&self, id: ComponentId) -> Option<&str> {
        self.editors.get(&id).map(|e| e.name.as_ref())
    }

    fn has(&self, id: ComponentId) -> bool {
        self.editors.contains_key(&id)
    }

    unsafe fn call_ui(&self, context: &mut EntityMut<'_>, id: ComponentId, ui: &mut Ui) {
        let editor = self.editors.get(&id).unwrap();
        let data = get_component_from_mut(context, id);

        if let Some(data) = data {
            (editor.ui)(ui, data)
        }
    }
}

#[inline]
unsafe fn get_component_from_mut(
    context: &mut EntityMut<'_>,
    component_id: ComponentId,
) -> Option<*mut u8> {
    let (entity, location) = (context.id(), context.location());
    let world = context.world();

    let archetype = &world.archetypes()[location.archetype_id];
    // SAFE: component_id exists and is therefore valid
    let component_info = world.components().get_info_unchecked(component_id);
    match component_info.storage_type() {
        StorageType::Table => {
            let table = &world.storages().tables[archetype.table_id()];
            let components = table.get_column(component_id)?;
            let table_row = archetype.entity_table_row(location.index);
            // SAFE: archetypes only store valid table_rows and the stored component type is T
            Some(components.get_data_unchecked(table_row))
        }
        StorageType::SparseSet => world
            .storages()
            .sparse_sets
            .get(component_id)
            .and_then(|sparse_set| sparse_set.get(entity)),
    }
}

fn ui_transform(
    ui: &mut Ui,
    translation: &mut bevy::prelude::Vec3,
    rotation: &mut bevy::prelude::Quat,
    scale: &mut bevy::prelude::Vec3,

    scope_id: impl std::hash::Hash,
) {
    use bevy::prelude::*;

    let frame = Frame::none().margin(egui::style::Margin::symmetric(2.0, 2.0));

    frame.show(ui, |ui| {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(2.0, 2.0);

            ui.columns(2, |ui| {
                ui[0].add(Label::new("translation").sense(Sense::click()));
                ui[1].columns(3, |ui| {
                    ui[0].add(DragValue::new(&mut translation.x).speed(0.1));
                    ui[1].add(DragValue::new(&mut translation.y).speed(0.1));
                    ui[2].add(DragValue::new(&mut translation.z).speed(0.1));
                });
            });

            let id = Id::new((ui.id(), scope_id, "#euler"));
            let mut euler: EulerRot = ui.ctx().data().get_temp(id).unwrap_or(EulerRot::XYZ);

            macro_rules! selectable_euler {
                ($ui:expr, $current:ident, $selected:ident) => {{
                    let mut response = $ui.selectable_label(
                        matches!($current, EulerRot::$selected),
                        format!("{:?}", EulerRot::$selected),
                    );
                    if response.clicked() {
                        $current = EulerRot::$selected;
                        response.mark_changed();
                        $ui.close_menu();
                    }
                }};
            }

            ui.columns(2, |ui| {
                let label = match euler {
                    EulerRot::XYZ => String::from("rotation"),
                    _ => format!("rotation {:?}", euler),
                };
                let response = ui[0].add(Label::new(label).sense(Sense::click()));

                response.context_menu(|ui| {
                    selectable_euler!(ui, euler, XYZ);
                    selectable_euler!(ui, euler, XZY);
                    selectable_euler!(ui, euler, YXZ);
                    selectable_euler!(ui, euler, YZX);
                    selectable_euler!(ui, euler, ZYX);
                    selectable_euler!(ui, euler, ZXY);
                });

                ui[1].columns(3, |ui| {
                    let (mut a, mut b, mut c) = rotation.to_euler(euler);
                    ui[0].drag_angle(&mut a);
                    ui[1].drag_angle(&mut b);
                    ui[2].drag_angle(&mut c);
                    *rotation = Quat::from_euler(euler, a, b, c);
                });
            });

            ui.ctx().data().insert_temp(id, euler);

            ui.columns(2, |ui| {
                ui[0].add(Label::new("scale").sense(Sense::click()));
                ui[1].columns(3, |ui| {
                    ui[0].add(DragValue::new(&mut scale.x).speed(0.1));
                    ui[1].add(DragValue::new(&mut scale.y).speed(0.1));
                    ui[2].add(DragValue::new(&mut scale.z).speed(0.1));
                });
            });
        });
    });
}
