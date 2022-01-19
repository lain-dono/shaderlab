use iced_native::{
    event::{self, Event},
    layout, mouse, overlay, renderer, touch,
    {Clipboard, Element, Hasher, Layout, Length, Padding, Point, Rectangle, Shell, Widget},
};
use iced_wgpu::Renderer;
use std::{cell::Cell, hash::Hash};

pub struct Pad<'a, Message> {
    state: &'a mut State,
    content: Element<'a, Message, Renderer>,
    width: Length,
    height: Length,
    min_width: u32,
    min_height: u32,
    padding: Padding,

    on_press: Option<Message>,
    on_release: Option<Message>,

    interaction: mouse::Interaction,
}

impl<'a, Message: Clone> Pad<'a, Message> {
    /// Creates a new [`Pad`] with some local [`State`] and the given content.
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            state,
            content: content.into(),
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: Padding::new(5),

            on_press: None,
            on_release: None,

            interaction: mouse::Interaction::Pointer,
        }
    }

    /// Sets the width of the [`Pad`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Pad`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width of the [`Pad`].
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the minimum height of the [`Pad`].
    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the [`Padding`] of the [`Pad`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the [`Pad`] is pressed.
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Sets the message that will be produced when the [`Pad`] is released.
    pub fn on_release(mut self, msg: Message) -> Self {
        self.on_release = Some(msg);
        self
    }

    pub fn interaction(mut self, interaction: mouse::Interaction) -> Self {
        self.interaction = interaction;
        self
    }
}

/// The local state of a [`Pad`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    is_pressed: bool,
    bounds: Cell<Rectangle>,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> State {
        State::default()
    }

    pub fn bounds(&self) -> Rectangle {
        self.bounds.get()
    }
}

impl<'a, Message: Clone> Widget<Message, Renderer> for Pad<'a, Message> {
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits
            .min_width(self.min_width)
            .min_height(self.min_height)
            .width(self.width)
            .height(self.height)
            .pad(self.padding);

        let mut content = self.content.layout(renderer, &limits);
        content.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        let size = limits.resolve(content.size()).pad(self.padding);

        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) -> event::Status {
        if !self.state.is_pressed {
            if let event::Status::Captured = self.content.on_event(
                event.clone(),
                layout.children().next().unwrap(),
                cursor_position,
                renderer,
                clipboard,
                shell,
            ) {
                return event::Status::Captured;
            }
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_release.is_some() {
                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        self.state.is_pressed = true;

                        if let Some(on_press) = self.on_press.clone() {
                            shell.publish(on_press);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if let Some(on_release) = self.on_release.clone() {
                    let bounds = layout.bounds();

                    if self.state.is_pressed {
                        self.state.is_pressed = false;

                        if bounds.contains(cursor_position) {
                            shell.publish(on_release);
                        }

                        return event::Status::Captured;
                    }
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => self.state.is_pressed = false,
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        self.state.bounds.set(bounds);

        let content = &self.content;
        let layout = layout.children().next().unwrap();
        content.draw(renderer, style, layout, cursor_position, &bounds);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        self.state.bounds.set(bounds);

        let content = &self.content;
        let layout = layout.children().next().unwrap();
        let _ = content.mouse_interaction(layout, cursor_position, viewport, renderer);

        let is_mouse_over = bounds.contains(cursor_position);

        if is_mouse_over {
            self.interaction
        } else {
            mouse::Interaction::default()
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.content.hash_layout(state);
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content
            .overlay(layout.children().next().unwrap(), renderer)
    }
}

impl<'a, Message: 'a + Clone> From<Pad<'a, Message>> for Element<'a, Message, Renderer> {
    fn from(widget: Pad<'a, Message>) -> Self {
        Self::new(widget)
    }
}
