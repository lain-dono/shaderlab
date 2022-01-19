use crate::node::{Message, Pending, Type};
use crate::style;
use iced_graphics::backend::Text as _;
use iced_native::{
    alignment,
    event::{self, Event},
    layout, mouse, renderer, touch,
    {Clipboard, Element, Hasher, Layout, Length, Point, Rectangle, Size},
};
use iced_wgpu::{Primitive, Renderer};
use std::{cell::Cell, hash::Hash};

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PortId(pub usize);

impl ToString for PortId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct State {
    pub ty: Type,
    pub label: String,
    pub is_pressed: Cell<bool>,
    pub slot: Cell<Point>,
}

impl State {
    pub fn slot(&self) -> Point {
        self.slot.get()
    }

    pub fn new(label: impl ToString, ty: Type) -> Self {
        Self {
            label: label.to_string(),
            is_pressed: Default::default(),
            ty,
            slot: Default::default(),
        }
    }

    pub fn view(&self, on_press: Pending, is_set: bool) -> Element<Message, Renderer> {
        let on_press = on_press.translate(self.slot.get() - Point::ORIGIN);

        Element::new(Widget {
            state: self,
            on_press: Message::StartEdge(on_press),
            is_input: on_press.is_input(),
            size: style::FONT_SIZE,
            is_set,
        })
    }
}

struct Widget<'a, Message> {
    state: &'a State,
    on_press: Message,
    is_input: bool,
    size: u16,
    is_set: bool,
}

impl<'a, Message: Clone> iced_native::Widget<Message, Renderer> for Widget<'a, Message> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width()).height(self.height());

        let (width, height) = renderer.backend().measure(
            &self.state.label,
            f32::from(self.size),
            Default::default(),
            limits.max(),
        );

        let size = Size::new(width + height, height);
        let size = limits.resolve(size);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        messages: &mut iced_native::Shell<Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let center = Point::new(
            if self.is_input {
                bounds.x + bounds.height / 2.0
            } else {
                bounds.x - bounds.height / 2.0 + bounds.width
            },
            bounds.y + bounds.height / 2.0,
        );
        self.state.slot.set(center);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if bounds.contains(cursor_position) {
                    self.state.is_pressed.set(true);
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if self.state.is_pressed.get() {
                    self.state.is_pressed.set(false);

                    if bounds.contains(cursor_position) {
                        messages.publish(self.on_press.clone());
                    }

                    return event::Status::Captured;
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => self.state.is_pressed.set(false),
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let center = Point::new(
            if self.is_input {
                bounds.x + bounds.height / 2.0
            } else {
                bounds.x - bounds.height / 2.0 + bounds.width
            },
            bounds.y + bounds.height / 2.0,
        );
        self.state.slot.set(center);

        let pad = 3.5;
        let pad_set = 1.5;
        let bg = style::PORT_BACKGROUND;
        let fg = match self.state.ty {
            Type::Vector1 => style::PORT_VECTOR_1,
            Type::Vector2 => style::PORT_VECTOR_2,
            Type::Vector3 => style::PORT_VECTOR_3,
            Type::Vector4 => style::PORT_VECTOR_4,
        };
        //let (fg, bg) = (style::PORT_COLOR, );
        //let (border_color, background) = if self.is_input { (fg, bg) } else { (bg, fg) };
        let (border_color, background) = (fg, bg);

        renderer.draw_primitive(Primitive::Quad {
            bounds: Rectangle {
                x: center.x - pad,
                y: center.y - pad,
                width: pad * 2.0,
                height: pad * 2.0,
            },
            background: background.into(),
            border_radius: 100.0,
            border_width: 1.0,
            border_color,
        });

        renderer.draw_primitive(Primitive::Quad {
            bounds: Rectangle {
                x: center.x - pad_set,
                y: center.y - pad_set,
                width: pad_set * 2.0,
                height: pad_set * 2.0,
            },
            background: if self.is_set {
                border_color.into()
            } else {
                background.into()
            },
            border_radius: 100.0,
            border_width: 0.0,
            border_color,
        });

        renderer.draw_primitive(Primitive::Text {
            content: self.state.label.to_string(),
            size: f32::from(self.size),
            bounds: Rectangle {
                x: if self.is_input {
                    bounds.x + bounds.height
                } else {
                    bounds.x + bounds.width - bounds.height
                },
                y: bounds.y - 1.0,
                width: bounds.width - bounds.height,
                height: bounds.height,
            },
            color: style.text_color,
            font: Default::default(),
            horizontal_alignment: if self.is_input {
                alignment::Horizontal::Left
            } else {
                alignment::Horizontal::Right
            },
            vertical_alignment: alignment::Vertical::Top,
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.size.hash(state);
    }
}
