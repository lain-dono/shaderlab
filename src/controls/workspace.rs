use super::{Edge, Message, Pending};
use iced_graphics::canvas::{self, Canvas, Cursor, Frame, Geometry};
use iced_graphics::{Defaults, Primitive};
use iced_native::{
    event, layout, mouse, overlay, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Vector, Widget,
};
use iced_wgpu::Renderer;
use std::cell::Cell;

#[derive(Default)]
pub struct State {
    bounds: Cell<Rectangle>,
    pending: Option<Pending>,
    cache: canvas::Cache,
}

impl State {
    pub fn bounds(&self) -> Rectangle {
        self.bounds.get()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
    }

    pub fn pending(&self) -> Option<Pending> {
        self.pending
    }

    pub fn start(&mut self, pending: Pending) {
        self.pending = Some(pending);
    }

    pub fn end(&mut self) -> Option<Pending> {
        self.pending.take()
    }
}

struct Item<'a> {
    position: Point,
    widget: Element<'a, Message, Renderer>,
}

pub struct Workspace<'a> {
    bounds: &'a Cell<Rectangle>,
    canvas: Element<'a, Message, Renderer>,
    offset: Vector,
    children: Vec<Item<'a>>,
}

impl<'a> Workspace<'a> {
    pub fn new(state: &'a mut State, curves: &'a [Edge], cancel: Message) -> Self {
        let pending = &mut state.pending;
        let cache = &mut state.cache;
        let canvas: Element<_, _> = Canvas::new(Bezier {
            pending,
            cache,
            curves,
            cancel,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        Self {
            bounds: &state.bounds,
            canvas,
            offset: Vector::new(0.0, 0.0),
            children: vec![],
        }
    }

    pub fn push(
        mut self,
        position: Point,
        widget: impl Into<Element<'a, Message, Renderer>>,
    ) -> Self {
        let widget = widget.into();
        self.children.push(Item { position, widget });
        self
    }
}

impl<'a> Widget<Message, Renderer> for Workspace<'a> {
    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Fill
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let size = limits
            .width(Length::Fill)
            .height(Length::Fill)
            .resolve(Size::ZERO);

        let child_limits = layout::Limits::new(Size::ZERO, size);

        let children = std::iter::once(self.canvas.layout(renderer, &child_limits))
            .chain(self.children.iter().map(|child| {
                let mut node = child.widget.layout(renderer, &child_limits);
                node.move_to(child.position);
                node
            }))
            .collect();

        layout::Node::with_children(size, children)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.canvas.hash_layout(state);

        self.offset.x.to_ne_bytes().hash(state);
        self.offset.y.to_ne_bytes().hash(state);

        for child in &self.children {
            child.position.x.to_ne_bytes().hash(state);
            child.position.y.to_ne_bytes().hash(state);
            child.widget.hash_layout(state);
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        let mut children = layout.children();
        let canvas_layout = children.next();

        if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
            messages.push(Message::Move(position))
        }

        let mut status = event::Status::Ignored;

        for (child, layout) in self.children.iter_mut().zip(children) {
            status = status.merge(child.widget.on_event(
                event.clone(),
                layout,
                cursor_position,
                renderer,
                clipboard,
                messages,
            ));
            if matches!(status, event::Status::Captured) {
                return status;
            }
        }

        canvas_layout.map_or(status, |layout| {
            status.merge(self.canvas.on_event(
                event,
                layout,
                cursor_position,
                renderer,
                clipboard,
                messages,
            ))
        })
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        let bounds = layout.bounds();
        self.bounds.set(bounds);

        let mut mouse_interaction = mouse::Interaction::default();

        let mut children = layout.children();

        let canvas = children.next().map(|layout| {
            let (primitive, _new_mouse_interaction) =
                self.canvas
                    .draw(renderer, defaults, layout, cursor_position, viewport);

            //mouse_interaction = mouse_interaction.max(new_mouse_interaction);

            primitive
        });

        let primitives: Vec<_> = canvas
            .into_iter()
            .chain(self.children.iter().zip(children).map(|(child, layout)| {
                let (primitive, new_mouse_interaction) =
                    child
                        .widget
                        .draw(renderer, defaults, layout, cursor_position, viewport);

                mouse_interaction = mouse_interaction.max(new_mouse_interaction);

                primitive
            }))
            .collect();

        let primitive = Primitive::Clip {
            bounds,
            offset: Default::default(),
            content: Box::new(Primitive::Group { primitives }),
        };

        (primitive, mouse_interaction)
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        let mut children = layout.children();

        let layout = children.next();
        let canvas = &mut self.canvas;
        if let Some(el) = layout.and_then(move |layout| canvas.overlay(layout)) {
            return Some(el);
        }

        self.children
            .iter_mut()
            .zip(children)
            .find_map(|(child, layout)| child.widget.overlay(layout))
    }
}

impl<'a> From<Workspace<'a>> for Element<'a, Message, Renderer> {
    fn from(widget: Workspace<'a>) -> Self {
        Element::new(widget)
    }
}

struct Bezier<'a, Message> {
    pending: &'a Option<Pending>,
    cache: &'a mut canvas::Cache,
    curves: &'a [Edge],
    cancel: Message,
}

impl<'a, Message: Clone> canvas::Program<Message> for Bezier<'a, Message> {
    fn update(
        &mut self,
        event: canvas::event::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        if !cursor.is_over(&bounds) {
            return (event::Status::Ignored, None);
        }

        if let canvas::Event::Mouse(mouse_event) = event {
            let message = match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => Some(self.cancel.clone()),
                _ => None,
            };

            (event::Status::Captured, message)
        } else {
            (event::Status::Ignored, None)
        }
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let content = self.cache.draw(bounds.size(), |frame: &mut Frame| {
            Edge::draw_all(self.curves, frame)
        });

        if let Some(pending) = &self.pending {
            let pending_curve = pending.draw(bounds, cursor);
            vec![content, pending_curve]
        } else {
            vec![content]
        }
    }

    fn mouse_interaction(&self, bounds: Rectangle, cursor: Cursor) -> mouse::Interaction {
        if cursor.is_over(&bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}
