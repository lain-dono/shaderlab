use iced_native::{
    event::{self, Event},
    layout, mouse, overlay, renderer, touch,
    {Clipboard, Element, Hasher, Layout, Length, Point, Rectangle, Shell, Widget},
};
use iced_wgpu::Renderer;

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
        shell: &mut Shell<Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.on_event(
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                if layout.bounds().contains(cursor_position) {
                    shell.publish(self.message.clone());
                }
                event::Status::Captured
            }
            _ => event::Status::Ignored,
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content
            .draw(renderer, style, layout, cursor_position, viewport);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        let _ = self
            .content
            .mouse_interaction(layout, cursor_position, viewport, renderer);

        if bounds.contains(cursor_position) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.content.hash_layout(state);
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content.overlay(layout, renderer)
    }
}

impl<'a, Message: 'a + Clone> From<PressPad<'a, Message>> for Element<'a, Message, Renderer> {
    fn from(widget: PressPad<'a, Message>) -> Element<'a, Message, Renderer> {
        Element::new(widget)
    }
}
