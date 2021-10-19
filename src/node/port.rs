use crate::node::{Message, Pending};
use crate::style;
use iced_graphics::backend::Text as _;
use iced_native::{
    alignment,
    event::{self, Event},
    layout, mouse, touch, {Clipboard, Element, Hasher, Layout, Length, Point, Rectangle, Size},
};
use iced_wgpu::{Defaults, Primitive, Renderer};
use std::{cell::Cell, hash::Hash};

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PortId(pub usize);

impl ToString for PortId {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub struct PortState {
    label: String,
    is_pressed: bool,
    slot: Cell<Point>,
}

impl PortState {
    pub fn slot(&self) -> Point {
        self.slot.get()
    }

    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            is_pressed: false,
            slot: Default::default(),
        }
    }

    pub fn view(&mut self, on_press: Pending) -> Element<Message, Renderer> {
        let on_press = on_press.translate(self.slot.get() - Point::ORIGIN);

        Element::new(Widget {
            is_pressed: &mut self.is_pressed,
            slot: &self.slot,
            content: &self.label,
            on_press: Message::StartEdge(on_press),
            is_input: on_press.is_input(),
            size: style::FONT_SIZE,
        })
    }
}

struct Widget<'a, Message> {
    is_pressed: &'a mut bool,
    slot: &'a Cell<Point>,
    content: &'a str,
    on_press: Message,
    is_input: bool,
    size: u16,
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
            self.content,
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
        messages: &mut Vec<Message>,
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
        self.slot.set(center);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if bounds.contains(cursor_position) {
                    *self.is_pressed = true;
                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if *self.is_pressed {
                    *self.is_pressed = false;

                    if bounds.contains(cursor_position) {
                        messages.push(self.on_press.clone());
                    }

                    return event::Status::Captured;
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => *self.is_pressed = false,
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        defaults: &Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        let bounds = layout.bounds();

        let is_hover = bounds.contains(cursor_position);

        let mouse_interaction = if is_hover {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        };

        let center = Point::new(
            if self.is_input {
                bounds.x + bounds.height / 2.0
            } else {
                bounds.x - bounds.height / 2.0 + bounds.width
            },
            bounds.y + bounds.height / 2.0,
        );
        self.slot.set(center);

        let pad = 3.5;
        let pad_set = 1.5;
        let (fg, bg) = (style::PORT_COLOR, style::PORT_BACKGROUND);
        //let (border_color, background) = if self.is_input { (fg, bg) } else { (bg, fg) };
        let (border_color, background) = (fg, bg);

        let slot = Primitive::Quad {
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
        };

        let is_set = !is_hover;

        let slot_set = Primitive::Quad {
            bounds: Rectangle {
                x: center.x - pad_set,
                y: center.y - pad_set,
                width: pad_set * 2.0,
                height: pad_set * 2.0,
            },
            background: if is_set {
                border_color.into()
            } else {
                background.into()
            },
            border_radius: 100.0,
            border_width: 0.0,
            border_color,
        };

        let label = Primitive::Text {
            content: self.content.to_string(),
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
            color: defaults.text.color,
            font: Default::default(),
            horizontal_alignment: if self.is_input {
                alignment::Horizontal::Left
            } else {
                alignment::Horizontal::Right
            },
            vertical_alignment: alignment::Vertical::Top,
        };

        let primitives = vec![slot, slot_set, label];

        (Primitive::Group { primitives }, mouse_interaction)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.size.hash(state);
    }
}
