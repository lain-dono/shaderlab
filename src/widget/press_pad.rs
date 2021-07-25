use iced_native::{
    event::{self, Event},
    layout, mouse, overlay, touch,
    {Clipboard, Element, Hasher, Layout, Length, Point, Rectangle, Widget},
};
use iced_wgpu::{Defaults, Primitive, Renderer};

pub struct PressPad<'a, Message> {
    content: Element<'a, Message, Renderer>,
    message: Message,
}

impl<'a, Message: Clone> PressPad<'a, Message> {
    /// Creates a new [`Pad`] with some local [`State`] and the given content.
    pub fn new<E>(content: E, message: Message) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Self {
            content: content.into(),
            message,
        }
    }
}

impl<'a, Message: Clone> Widget<Message, Renderer> for PressPad<'a, Message> {
    fn width(&self) -> Length {
        self.content.width()
    }

    fn height(&self) -> Length {
        self.content.height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.layout(renderer, limits)
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
        if let event::Status::Captured = self.content.on_event(
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    messages.push(self.message.clone());
                }
                event::Status::Captured
            }
            _ => event::Status::Ignored,
        }
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

        let (content, _) = self
            .content
            .draw(renderer, defaults, layout, cursor_position, viewport);

        (
            content,
            if bounds.contains(cursor_position) {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.content.hash_layout(state);
    }

    fn overlay(&mut self, layout: Layout<'_>) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content.overlay(layout)
    }
}

impl<'a, Message: 'a + Clone> From<PressPad<'a, Message>> for Element<'a, Message, Renderer> {
    fn from(widget: PressPad<'a, Message>) -> Element<'a, Message, Renderer> {
        Element::new(widget)
    }
}
