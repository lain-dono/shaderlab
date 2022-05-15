use crate::style::Style;
use bevy::reflect::{FromType, Reflect};
use egui::style::Margin;
use egui::*;
use std::borrow::Cow;

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
    pub fn skip(&self) -> bool {
        (self.skip)()
    }

    pub fn ui(&self, ui: &mut Ui, style: &Style, reflect: &mut dyn Reflect) {
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

pub fn reflect_component_editor(
    ui: &mut egui::Ui,
    style: &Style,
    reflect: &mut dyn Reflect,
    icon: char,
    name: &str,
) {
    let id = Id::new((reflect.type_name(), "#component_header"));
    let mut state = State::load(ui.ctx(), id);

    if component_header(Some(&mut state), ui, style, icon, name, |_| ()) {
        let margin = Margin {
            left: 6.0,
            right: 2.0,
            top: 4.0,
            bottom: 6.0,
        };
        Frame::none().margin(margin).show(ui, |ui| {
            crate::field::reflect(ui, reflect);
        });
    }

    state.store(ui.ctx(), id);
}

pub fn component_header(
    state: Option<&mut State>,
    ui: &mut egui::Ui,
    style: &Style,
    icon: char,
    name: &str,
    extra: impl FnOnce(&mut Ui),
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

    let dots_rect = ui.painter().text(
        dots_pos,
        Align2::CENTER_CENTER,
        crate::icon::THREE_DOTS,
        FontId::proportional(16.0),
        text_color,
    );

    {
        let left_x = egui::lerp(rect.min.x..=rect.max.x, crate::field::PERCENT);
        let rect = rect.intersect(Rect::everything_right_of(left_x + 7.0));
        let rect = rect.intersect(Rect::everything_left_of(dots_rect.min.x));
        let rect = rect.shrink2(vec2(0.0, 1.0));

        let layout = Layout::top_down(Align::Min);
        let mut ui = ui.child_ui(rect, layout);
        extra(&mut ui)
    }

    if let Some(state) = state {
        if response.clicked() {
            state.toggle(ui);
        }
        ui.painter().text(
            tri_pos,
            Align2::CENTER_CENTER,
            if state.open {
                crate::icon::DISCLOSURE_TRI_DOWN
            } else {
                crate::icon::DISCLOSURE_TRI_RIGHT
            },
            FontId::proportional(16.0),
            tri_color,
        );
        state.open
    } else {
        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    open: bool,
}

impl State {
    pub fn load(ctx: &Context, id: Id) -> Self {
        ctx.data().get_temp(id).unwrap_or(Self { open: true })
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data().insert_temp(id, self);
    }

    pub fn toggle(&mut self, ui: &Ui) {
        self.open = !self.open;
        ui.ctx().request_repaint();
    }
}
