use crate::node::{
    self, Edge, Input, Message, Node, NodeId, NodeMap, NodeWidget, Output, Pending, PortId,
};
use iced_graphics::canvas::{self, Canvas, Cursor, Frame, Geometry};
use iced_graphics::{Defaults, Primitive};
use iced_native::{
    event, layout, mouse, overlay, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Vector, Widget,
};
use iced_wgpu::Renderer;
use std::cell::Cell;

pub fn fix_name(s: &str) -> String {
    s.chars()
        .map(|c| if !c.is_ascii_alphanumeric() { '_' } else { c })
        .collect()
}

#[derive(Default)]
pub struct State {
    bounds: Cell<Rectangle>,
    cache: canvas::Cache,

    nodes: NodeMap,
    edges: Vec<Edge>,

    pending: Option<Pending>,
    pub drag: Option<NodeId>,
    pub drag_last: Point,
}

impl State {
    pub fn with_nodes(nodes: NodeMap) -> Self {
        Self {
            nodes,
            drag: None,
            drag_last: Point::ORIGIN,
            ..Self::default()
        }
    }

    pub fn update_node(&mut self, node: NodeId, message: Box<dyn crate::node::DynMessage>) {
        self.nodes[node].event(node::Event::Dynamic(message));
    }

    pub fn set_port_default(&mut self, node: NodeId, port: PortId, value: [f64; 4]) {
        self.nodes[node].set_port_default(port, value)
    }

    pub fn add_node(&mut self, node: Box<dyn Node>) {
        log::info!("add node {:?}", node);
        let bounds = self.bounds();
        let position = Point::ORIGIN + (bounds.center() - bounds.position());

        self.nodes
            .insert_with_key(|id| NodeWidget::new(id, position, node));
    }

    pub fn remove_node(&mut self, node: NodeId) {
        let removed = self.edges.iter().filter(|edge| edge.has_node(node));

        for &edge in removed {
            self.nodes[edge.input()].event(node::Event::AttachInput(edge.input().port(), None));
            self.nodes[edge.output()].event(node::Event::AttachOutput(edge.output().port(), None));
        }

        self.edges.retain(|edge| !edge.has_node(node));

        self.nodes.remove(node);
        self.request_redraw();
    }

    pub fn start_edge(&mut self, from: Pending) {
        let base_offset = Point::ORIGIN - self.bounds().position();
        let from = from.translate(base_offset);

        if let Some(to) = self.pending {
            if let Some(edge) = Edge::new(from, to).filter(Edge::not_same_node) {
                if let Some(index) = self.edges.iter().position(|e| e.eq_node_port(&edge)) {
                    log::info!("remove {}", edge);
                    self.edges.remove(index);
                    self.nodes[edge.input()]
                        .event(node::Event::AttachInput(edge.input().port(), None));
                    self.nodes[edge.output()]
                        .event(node::Event::AttachOutput(edge.output().port(), None));
                // only one edge for input port and not cycle
                } else if self.edges.iter().all(|e| e.input() != edge.input())
                    && self.find_cycle(edge)
                {
                    log::info!("create {}", edge);
                    self.edges.push(edge);

                    self.nodes[edge.input()].event(node::Event::AttachInput(
                        edge.input().port(),
                        Some(edge.output()),
                    ));
                    self.nodes[edge.output()].event(node::Event::AttachOutput(
                        edge.output().port(),
                        Some(edge.input()),
                    ));
                }
            }

            self.end();
            self.request_redraw();
        } else {
            log::info!("start {}", from);
            self.pending = Some(from);
        }
    }

    pub fn move_node(&mut self, id: NodeId, delta: Vector) {
        let node = &mut self.nodes[id];
        node.set_position(node.position() + delta);
        self.fix_node_position(id);
    }

    fn find_cycle(&self, edge: Edge) -> bool {
        let mut graph = crate::graph::Graph::default();
        for edge in &self.edges {
            graph.add_edge(edge.output().node(), edge.input().node());
        }
        graph.add_edge(edge.output().node(), edge.input().node());
        !graph.is_reachable(edge.input().node(), edge.output().node())
    }

    pub fn fix_node_position(&mut self, node: NodeId) {
        let base_offset = Point::ORIGIN - self.bounds().position();

        let node = if let Some(node) = self.nodes.get(node) {
            node
        } else {
            log::warn!("can't find node: {:?}", node);
            return;
        };

        for (port, input) in node.inputs() {
            let position = input.slot();
            for edge in &mut self.edges {
                if edge.input() == Input::new(node.id(), port) {
                    edge.set_input_position(position + base_offset);
                }
            }
        }

        for (port, output) in node.outputs() {
            let position = output.slot();
            for edge in &mut self.edges {
                if edge.output() == Output::new(node.id(), port) {
                    edge.set_output_position(position + base_offset);
                }
            }
        }

        self.request_redraw();
    }

    pub fn try_traverse(&mut self) -> Option<String> {
        let module = self
            .nodes
            .values()
            .find_map(|node| node.downcast_ref::<crate::node::master::Master>())
            .and_then(|master| master.entry(&self.nodes));

        if let Some(module) = module {
            log::trace!("{:#?}", module.module());
            let source = module.build();
            println!("{}", source);
            Some(source)
        } else {
            None
        }
    }
}

impl State {
    pub fn bounds(&self) -> Rectangle {
        self.bounds.get()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear()
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
    pub fn new(state: &'a mut State) -> Self {
        let pending = &mut state.pending;
        let cache = &mut state.cache;
        let canvas: Element<_, _> = Canvas::new(Bezier {
            pending,
            cache,
            curves: &state.edges,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        let children = state
            .nodes
            .values_mut()
            .map(|node| Item {
                position: node.position(),
                widget: node.widget(),
            })
            .collect();

        Self {
            bounds: &state.bounds,
            canvas,
            offset: Vector::new(0.0, 0.0),
            children,
        }
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
            messages.push(Message::DragMove(position))
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

                let primitives = vec![primitive];

                Primitive::Clip {
                    bounds,
                    offset: Default::default(),
                    content: Box::new(Primitive::Group { primitives }),
                }
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

struct Bezier<'a> {
    pending: &'a Option<Pending>,
    cache: &'a mut canvas::Cache,
    curves: &'a [Edge],
}

impl<'a> canvas::Program<Message> for Bezier<'a> {
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
                mouse::Event::ButtonPressed(mouse::Button::Left) => Some(Message::CancelEdge),
                _ => None,
            };

            (event::Status::Captured, message)
        } else {
            (event::Status::Ignored, None)
        }
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let content = self.cache.draw(bounds.size(), |frame: &mut Frame| {
            Edge::draw(frame, self.curves)
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
