use crate::app::{Style, TabInner};
use crate::global::{Global, Icon};
use bevy::prelude::{Entity, Name};
use egui::style::Margin;
use egui::text::LayoutJob;
use egui::widgets::TextEdit;
use egui::*;

#[derive(Default)]
pub struct Inspector {
    lock: Option<Entity>,
}

impl TabInner for Inspector {
    fn ui(&mut self, ui: &mut Ui, style: &Style, global: &mut Global) {
        let rect = ui.available_rect_before_wrap();
        ui.painter()
            .rect_filled(rect, 0.0, Color32::from_gray(0x28));

        let entity = self
            .lock
            .or(global.selected)
            .and_then(|entity| global.world.get_entity_mut(entity));

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
}
