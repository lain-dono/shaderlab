use iced_native::{Background, Vector};
use iced_wgpu::{wgpu, widget, Color};

const fn rgb(value: u32) -> Color {
    rgba(value, 1.0)
}

const fn rgba(value: u32, a: f32) -> Color {
    let [b, g, r, _a] = value.to_le_bytes();
    let [b, g, r, _a] = [b as f32, g as f32, r as f32, _a as f32];
    let [b, g, r, _a] = [b / 255.0, g / 255.0, r / 255.0, _a / 255.0];
    Color { r, g, b, a }
}

pub fn to_clear_color(color: Color) -> iced_wgpu::wgpu::Color {
    // As described in: https://en.wikipedia.org/wiki/SRGB#The_reverse_transformation
    fn linear_component(u: f64) -> f64 {
        if u < 0.04045 {
            u / 12.92
        } else {
            ((u + 0.055) / 1.055).powf(2.4)
        }
    }

    let Color { r, g, b, a } = color;
    wgpu::Color {
        r: linear_component(r as f64),
        g: linear_component(g as f64),
        b: linear_component(b as f64),
        a: a as f64,
    }
}

pub const PORT_BOOL: Color = rgb(0x9481e6);
pub const PORT_STRUCT: Color = rgb(0xc8c8c8);
pub const PORT_IMAGE: Color = rgb(0xff8b8b);
pub const PORT_MATRIX: Color = rgb(0x8fc1df);

pub const PORT_VECTOR_1: Color = rgb(0x84e4e7);
pub const PORT_VECTOR_2: Color = rgb(0x9aef92);
pub const PORT_VECTOR_3: Color = rgb(0xf6ff9a);
pub const PORT_VECTOR_4: Color = rgb(0xfbcbf4);

pub const WORKSPACE_BG: Color = rgb(0x202020);

//pub const NODE_BACKGROUND: Color = rgba(0x333333, 0.95);
pub const NODE_TEXT: Color = rgb(0xCCCCCC);
pub const NODE_BACKGROUND: Color = rgba(0x333333, 0.90);
pub const NODE_BORDER: Color = rgba(0x1C1C1C, 0.90);

pub const SLIDER_RAIL: Color = rgb(0x5e5e5e);
pub const SLIDER_HANDLE_ACTIVE: Color = rgb(0x999999);
pub const SLIDER_HANDLE_HOVER: Color = rgb(0xEAEAEA);

pub const FONT_SIZE: u16 = 14;

pub const PORT_COLOR: Color = PORT_VECTOR_1;
pub const PORT_BACKGROUND: Color = rgba(0x333333, 0.90);

pub const SELECTION: Color = rgb(0x44c0ff);

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

pub struct Transparent;

impl widget::rule::StyleSheet for Transparent {
    fn style(&self) -> widget::rule::Style {
        widget::rule::Style {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            fill_mode: widget::rule::FillMode::Full,
            ..Default::default()
        }
    }
}

pub struct NodeBorder;

impl widget::container::StyleSheet for NodeBorder {
    fn style(&self) -> widget::container::Style {
        widget::container::Style {
            background: Some(NODE_BORDER.into()),
            border_width: 0.0,
            border_radius: 3.0,
            ..Default::default()
        }
    }
}

pub struct Node;

impl Node {
    pub fn input<'a, M: Clone, F: 'static + Fn(String) -> M>(
        state: &'a mut widget::text_input::State,
        placeholder: &str,
        value: &str,
        on_change: F,
    ) -> widget::text_input::TextInput<'a, M> {
        widget::text_input::TextInput::new(state, placeholder, value, on_change)
            .padding([1, 3])
            .style(Self)
            .size(FONT_SIZE - 2)
    }

    pub fn slider<'a, T: Copy + From<u8> + PartialOrd, M: Clone, F: 'static + Fn(T) -> M>(
        state: &'a mut crate::widget::slider::State,
        range: std::ops::RangeInclusive<T>,
        value: T,
        on_change: F,
    ) -> crate::widget::Slider<'a, T, M> {
        crate::widget::Slider::new(state, range, value, on_change)
            .style(Self)
            .height(FONT_SIZE)
    }
}

impl crate::widget::slider::StyleSheet for Node {
    fn active(&self) -> crate::widget::slider::Style {
        crate::widget::slider::Style {
            rail_colors: (SLIDER_RAIL, Color::TRANSPARENT),
            handle: crate::widget::slider::Handle {
                shape: crate::widget::slider::HandleShape::Circle { radius: 4.5 },
                color: SLIDER_HANDLE_ACTIVE,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self) -> crate::widget::slider::Style {
        crate::widget::slider::Style {
            rail_colors: (SLIDER_RAIL, Color::TRANSPARENT),
            handle: crate::widget::slider::Handle {
                shape: crate::widget::slider::HandleShape::Circle { radius: 4.5 },
                color: SLIDER_HANDLE_HOVER,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn dragging(&self) -> crate::widget::slider::Style {
        crate::widget::slider::Style {
            rail_colors: (SLIDER_RAIL, Color::TRANSPARENT),
            handle: crate::widget::slider::Handle {
                shape: crate::widget::slider::HandleShape::Circle { radius: 4.5 },
                color: SLIDER_HANDLE_HOVER,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

impl widget::container::StyleSheet for Node {
    fn style(&self) -> widget::container::Style {
        widget::container::Style {
            text_color: Some(NODE_TEXT),
            background: Some(NODE_BACKGROUND.into()),
            //border_radius: 4.0,
            border_width: 0.0,
            border_color: NODE_BORDER,
            ..Default::default()
        }
    }
}

impl widget::rule::StyleSheet for Node {
    fn style(&self) -> widget::rule::Style {
        widget::rule::Style {
            color: NODE_BORDER,
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
            background: rgb(0x2a2a2a).into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: rgb(0x212121),
        }
    }

    fn hovered(&self) -> widget::text_input::Style {
        widget::text_input::Style {
            background: rgb(0x2a2a2a).into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: rgb(0x656565),
        }
    }

    fn focused(&self) -> widget::text_input::Style {
        widget::text_input::Style {
            background: rgb(0x2a2a2a).into(),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: SELECTION,
        }
    }

    fn placeholder_color(&self) -> Color {
        NODE_TEXT
    }

    fn value_color(&self) -> Color {
        NODE_TEXT
    }

    fn selection_color(&self) -> Color {
        SELECTION
    }
}
