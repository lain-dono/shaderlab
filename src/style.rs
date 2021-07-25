use iced_native::{Background, Vector};
use iced_wgpu::{widget, Color};

pub const FONT_SIZE: u16 = 14;

pub const PORT_COLOR: Color = Color {
    r: 0.29411766,
    g: 0.57254905,
    b: 0.9529412,
    a: 1.0,
};

pub const SIDEBAR_BG: Color = Color {
    r: 0.153,
    g: 0.153,
    b: 0.153,
    a: 1.0,
};

pub struct Sidebar;

impl widget::container::StyleSheet for Sidebar {
    fn style(&self) -> widget::container::Style {
        widget::container::Style {
            text_color: Some(Color::WHITE),
            //background: Some(Color::from_rgb8(0x33, 0x33, 0x33).into()),
            background: Some(SIDEBAR_BG.into()),
            //border_radius: f32,
            //border_width: f32,
            //border_color: Color,
            ..Default::default()
        }
    }
}

pub struct Node;

impl widget::container::StyleSheet for Node {
    fn style(&self) -> widget::container::Style {
        widget::container::Style {
            text_color: Some(Color::from_rgb8(0xCC, 0xCC, 0xCC)),
            background: Some(Color::from_rgba8(0x33, 0x33, 0x33, 0.95).into()),
            //border_radius: 4.0,
            //border_width: f32,
            //border_color: Color,
            ..Default::default()
        }
    }
}

impl widget::rule::StyleSheet for Node {
    fn style(&self) -> widget::rule::Style {
        widget::rule::Style {
            //color: Color,
            //width: u16,
            //radius: f32,
            fill_mode: widget::rule::FillMode::Full,
            ..Default::default()
        }
    }
}

impl widget::button::StyleSheet for Node {
    fn active(&self) -> widget::button::Style {
        widget::button::Style {
            //shadow_offset: Vector,
            //background: Option<Background>,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::WHITE,
            text_color: Color::WHITE,
            ..Default::default()
        }
    }

    fn hovered(&self) -> widget::button::Style {
        let active = self.active();

        widget::button::Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self) -> widget::button::Style {
        widget::button::Style {
            shadow_offset: Vector::default(),
            ..self.active()
        }
    }

    fn disabled(&self) -> widget::button::Style {
        let active = self.active();

        widget::button::Style {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}

impl widget::text_input::StyleSheet for Node {
    fn active(&self) -> widget::text_input::Style {
        widget::text_input::Style {
            background: Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: Color::BLACK,
        }
    }
    fn focused(&self) -> widget::text_input::Style {
        widget::text_input::Style {
            background: Color::TRANSPARENT.into(),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: Color::BLACK,
        }
    }
    fn placeholder_color(&self) -> Color {
        Color::WHITE
    }
    fn value_color(&self) -> Color {
        Color::WHITE
    }
    fn selection_color(&self) -> Color {
        PORT_COLOR
    }
}
