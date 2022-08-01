use self::graph::Graph;
use ahash::AHashSet;
use slotmap::SlotMap;

mod graph;
mod link;
mod node;
mod port;
pub mod preview;

pub use self::{
    link::{Link, LinkBezier, LinkData, Slot},
    node::{Node, NodeBuilder, NodeData, NodeInteraction},
    port::Direction::{Input, Output},
    port::Stage::{Fragment, Vertex},
    port::{Data, Direction, InputDefault, InputDefaultType, Port, PortData, Stage},
    preview::{Preview, PreviewBuilder},
};

#[derive(Clone, Copy, Default, PartialEq)]
pub struct ButtonState {
    pub pressed: bool,
    pub released: bool,
    pub dragging: bool,
}

impl ButtonState {
    pub fn update(&mut self, down: bool) -> bool {
        let old = *self;
        self.released = (self.pressed || self.dragging) && !down;
        self.dragging = (self.pressed || self.dragging) && down;
        self.pressed = down && !(self.pressed || self.dragging);
        old != *self || self.dragging
    }
}

#[derive(Default)]
pub struct InputState {
    pub pointer: egui::Pos2,
    pub mouse: [ButtonState; 3],
}

impl InputState {
    pub fn update(&mut self, input: &egui::InputState) -> bool {
        self.pointer = input.pointer.hover_pos().unwrap_or(self.pointer);
        let primary = input.pointer.button_down(egui::PointerButton::Primary);
        let secondary = input.pointer.button_down(egui::PointerButton::Secondary);
        let middle = input.pointer.button_down(egui::PointerButton::Middle);

        let a = self.mouse[0].update(primary);
        let b = self.mouse[1].update(secondary);
        let c = self.mouse[2].update(middle);
        a || b || c
    }
}

#[derive(Default)]
pub struct Storage {
    pub nodes: SlotMap<Node, NodeData>,
    pub ports: SlotMap<Port, PortData>,
    pub links: SlotMap<Link, LinkData>,
}

impl Storage {
    pub fn link(&mut self, min: Port, max: Port) -> Link {
        Self::link_impl(&mut self.ports, &mut self.links, min, max)
    }

    pub fn unlink(&mut self, link: Link) -> Option<LinkData> {
        Self::unlink_impl(&mut self.ports, &mut self.links, link)
    }

    pub fn spawn<T, F, B>(&mut self, title: T, width: f32, builder: F) -> Node
    where
        T: Into<String>,
        B: PreviewBuilder + 'static,
        F: for<'a> FnOnce(&mut NodeBuilder<'a>, Node) -> B,
    {
        let mut node_builder = NodeBuilder {
            storage: crate::fuck_mut(self),
            inputs: Vec::new(),
            outputs: Vec::new(),
        };
        self.nodes.insert_with_key(|node| {
            let mut node = NodeData::new(title, width, builder(&mut node_builder, node));
            node.inputs = node_builder.inputs;
            node.outputs = node_builder.outputs;
            node
        })
    }

    pub fn despawn(&mut self, node: Node) {
        if let Some(removed) = self.nodes.remove(node) {
            tracing::info!("despawn {:?}", node);

            let inputs = removed.inputs.into_iter();
            let outputs = removed.outputs.into_iter();
            for port in inputs.chain(outputs) {
                if let Some(port) = self.ports.remove(port) {
                    for link in port.links {
                        self.unlink(link);
                    }
                }
            }
        }
    }

    fn link_impl(
        ports: &mut SlotMap<Port, PortData>,
        links: &mut SlotMap<Link, LinkData>,
        min: Port,
        max: Port,
    ) -> Link {
        tracing::info!("link {:?} -> {:?}", min, max);

        assert!(ports[min].is_output());
        assert!(ports[max].is_input());

        let link = links.insert(LinkData {
            min: Slot::new(ports[min].node, min),
            max: Slot::new(ports[max].node, max),
            shape: None,
        });

        ports[min].links.insert(link);
        ports[max].links.insert(link);

        link
    }

    fn unlink_impl(
        ports: &mut SlotMap<Port, PortData>,
        links: &mut SlotMap<Link, LinkData>,
        link: Link,
    ) -> Option<LinkData> {
        links.remove(link).map(|removed| {
            tracing::info!(
                "unlink {:?} [{:?} {:?}] -> [{:?} {:?}]",
                link,
                removed.min.node,
                removed.min.port,
                removed.max.node,
                removed.max.port,
            );

            if let Some(port) = ports.get_mut(removed.min.port) {
                port.links.remove(&link);
            }
            if let Some(port) = ports.get_mut(removed.max.port) {
                port.links.remove(&link);
            }

            removed
        })
    }

    fn should_link_snap(&mut self, graph: &mut Graph<Node>, current: Port, hovered: Port) -> bool {
        let start = &self.ports[current];
        let end = &self.ports[hovered];
        if start.node == end.node {
            return false;
        }

        if start.stage != end.stage {
            return false;
        }

        if !start.data.can_connect(end.data) {
            return false;
        }

        let any_link_with_port =
            |port: Port| -> bool { self.links.values().any(|link| link.has(port)) };

        let (output, input) = match (start.direction, end.direction) {
            (Input, Input) | (Output, Output) => return false,

            (Output, Input) if any_link_with_port(hovered) => return false,
            (Input, Output) if any_link_with_port(current) => return false,

            (Output, Input) => (start, end),
            (Input, Output) => (end, start),
        };

        graph.clear();

        for link in self.links.values() {
            let input = link.max.node;
            let output = link.min.node;
            graph.add_edge(output, input);
        }

        graph.add_edge(output.node, input.node);
        !graph.reachable(input.node, output.node)
    }
}

#[derive(Clone, Debug)]
enum Interaction {
    None,
    Panning,
    BoxSelection { start: egui::Pos2 },
    LinkCreation { current: Port },
    NodeCreation { position: egui::Pos2, first: bool },
}

pub struct Workspace {
    pub storage: Storage,
    pub dirty: bool,

    input: InputState,

    graph: Graph<Node>,
    search: String,

    interaction: Interaction,
    interaction_cache: Vec<NodeInteraction>,

    pub pan_offset: egui::Vec2,
    pan_button: egui::PointerButton,

    selection: AHashSet<Node>,
    selection_button: egui::PointerButton,
    selection_fill: egui::Color32,
    selection_outline: egui::Color32,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            storage: Storage::default(),
            dirty: true,

            interaction: Interaction::None,
            interaction_cache: Vec::new(),

            input: InputState::default(),

            graph: Graph::default(),
            search: String::new(),

            pan_offset: egui::Vec2::ZERO,
            pan_button: egui::PointerButton::Secondary,

            selection: AHashSet::default(),
            selection_button: egui::PointerButton::Primary,
            selection_fill: egui::Color32::from_rgba_unmultiplied(61, 133, 224, 30),
            selection_outline: egui::Color32::from_rgba_unmultiplied(61, 133, 224, 150),
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        self.dirty = self.input.update(&ctx.input());

        let mouse_is_over_area = ctx.available_rect().contains(self.input.pointer);

        match self.interaction {
            Interaction::None => {
                if mouse_is_over_area && ctx.input().key_pressed(egui::Key::Space) {
                    let position = self.input.pointer;
                    self.search.clear();
                    self.interaction = Interaction::NodeCreation {
                        position,
                        first: true,
                    };
                }

                if mouse_is_over_area
                    && !ctx.is_pointer_over_area()
                    && matches!(self.interaction, Interaction::None)
                {
                    let mut smallest_distance = f32::MAX;

                    let mut hovered_link = None;

                    for (key, link) in &mut self.storage.links {
                        let min = &self.storage.ports[link.min.port];
                        let max = &self.storage.ports[link.max.port];

                        let link_hover_distance = 10.0;

                        let bezier = LinkBezier::new(min.position, max.position);
                        let rect = bezier.containing_rect_for_bezier_curve(link_hover_distance);

                        if rect.contains(self.input.pointer) {
                            let distance = bezier.distance_to_cubic_bezier(self.input.pointer);
                            if distance < link_hover_distance && distance < smallest_distance {
                                smallest_distance = distance;
                                hovered_link.replace(key);
                            }
                        }
                    }

                    if self.input.mouse[self.pan_button as usize].pressed {
                        self.interaction = Interaction::Panning;
                    }

                    if self.input.mouse[self.selection_button as usize].pressed {
                        self.interaction = if let Some(link) = hovered_link {
                            let &LinkData { min, max, .. } = &self.storage.links[link];

                            let min_pos = self.storage.ports[min.port].position;
                            let max_pos = self.storage.ports[max.port].position;

                            let to_min = min_pos.distance(self.input.pointer);
                            let to_max = max_pos.distance(self.input.pointer);

                            let slot = if to_min > to_max { min } else { max };

                            self.storage.unlink(link);

                            Interaction::LinkCreation { current: slot.port }
                        } else {
                            Interaction::BoxSelection {
                                start: self.input.pointer,
                            }
                        }
                    }
                }
            }
            Interaction::Panning => {
                self.pan_offset += ctx.input().pointer.delta();
                if self.input.mouse[self.pan_button as usize].released {
                    self.interaction = Interaction::None;
                }
            }
            Interaction::BoxSelection { start } => self.box_selection(ctx, start),
            Interaction::LinkCreation { current } => {
                tracing::trace!("link creation {:?}", current)
            }

            Interaction::NodeCreation { position, first } => {
                self.node_creation(ctx, position, first)
            }
        }

        let painter = ctx.layer_painter(egui::LayerId::background());

        for link in self.storage.links.values_mut() {
            link.shape = Some(painter.add(egui::Shape::Noop));
        }

        let mut to_remove = None;
        let mut drag_delta = None;
        let mut hovered = None;
        let mut create_link = false;

        for (node_key, node) in &mut self.storage.nodes {
            let selected = self.selection.contains(&node_key);
            let selected = selected.then(|| self.selection_outline);

            node.draw(
                ctx,
                &mut self.storage.ports,
                node_key,
                self.pan_offset,
                selected,
                &mut self.interaction_cache,
            );

            for interaction in self.interaction_cache.drain(..) {
                match interaction {
                    NodeInteraction::Remove => to_remove = Some(node_key),
                    NodeInteraction::Drag(delta) => drag_delta = Some((node_key, delta)),
                    NodeInteraction::PortHovered(port) => hovered = Some(port),
                    NodeInteraction::LinkStart(mut current) => {
                        if let Interaction::None = self.interaction {
                            let port = &self.storage.ports[current];
                            if port.is_input() {
                                if let Some(link) = port.links.iter().next().copied() {
                                    let link = Storage::unlink_impl(
                                        &mut self.storage.ports,
                                        &mut self.storage.links,
                                        link,
                                    );
                                    current = link.unwrap().min.port;
                                }
                            }

                            self.interaction = Interaction::LinkCreation { current };
                        }
                    }
                    NodeInteraction::LinkEnd(_) => create_link = true,
                }
            }
        }

        if let Some((key, delta)) = drag_delta {
            if !self.selection.contains(&key) {
                self.selection.clear();
                self.selection.insert(key);
            }

            for &node in &self.selection {
                let node = match self.storage.nodes.get_mut(node) {
                    Some(node) => node,
                    None => continue,
                };

                node.position += delta;
                node.rect.min += delta;
                node.rect.max += delta;

                for &port in node.inputs.iter().chain(node.outputs.iter()) {
                    let port = &mut self.storage.ports[port];
                    port.position += delta;
                    port.rect.min += delta;
                    port.rect.max += delta;
                    if let Some(layer_id) = port.input_default.as_ref().and_then(|d| d.layer_id) {
                        ctx.translate_layer(layer_id, delta)
                    }
                }

                if let Some(layer_id) = node.layer {
                    ctx.translate_layer(layer_id, delta);
                }
            }
        }

        for link in self.storage.links.values_mut() {
            if let Some(idx) = link.shape.take() {
                let min = &self.storage.ports[link.min.port];
                let max = &self.storage.ports[link.max.port];
                let stroke = (2.0, min.data.color());
                if let Some(bezier) = LinkBezier::new(min.position, max.position).validate() {
                    painter.set(idx, bezier.draw(stroke));
                }
            }
        }

        if let Interaction::LinkCreation { current } = self.interaction {
            let id = egui::Id::new("link creation");
            let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
            let painter = ctx.layer_painter(layer_id);

            if hovered.is_none() {
                let hover_distance = 20.0;

                hovered = self
                    .storage
                    .ports
                    .iter()
                    .map(|(key, port)| (key, port.position.distance(self.input.pointer)))
                    .filter(|&(_, distance)| distance < hover_distance)
                    .max_by(|(_, a), (_, b)| f32::total_cmp(a, b))
                    .map(|(key, _)| key);
            }

            let snap_port = hovered.filter(|&hovered| {
                self.storage
                    .should_link_snap(&mut self.graph, current, hovered)
            });

            let mut max = self.input.pointer;

            if let Some(port) = snap_port {
                max = self.storage.ports[port].position;
            }

            let start = &self.storage.ports[current];
            let stroke = (2.0, start.data.color());

            let (min, max) = if start.is_output() {
                (start.position, max)
            } else {
                (max, start.position)
            };

            if create_link || self.input.mouse[self.selection_button as usize].released {
                self.interaction = Interaction::None;

                if let Some(snap_port) = snap_port {
                    if start.is_output() {
                        self.storage.link(current, snap_port);
                    } else {
                        self.storage.link(snap_port, current);
                    }
                }
            }

            if let Some(bezier) = LinkBezier::new(min, max).validate() {
                painter.add(bezier.draw(stroke));
            }
        }

        if let Some(node) = to_remove {
            self.storage.despawn(node);
        }
    }

    fn box_selection(&mut self, ctx: &egui::Context, start: egui::Pos2) {
        let btn = self.selection_button as usize;
        let current = self.input.pointer;

        let rect = egui::Rect::from_min_max(
            egui::pos2(start.x.min(current.x), start.y.min(current.y)),
            egui::pos2(start.x.max(current.x), start.y.max(current.y)),
        );

        let rect = ctx.available_rect().shrink(1.0).intersect(rect);

        self.selection.clear();
        for (key, node) in self.storage.nodes.iter() {
            if rect.intersects(node.rect) {
                self.selection.insert(key);
            }
        }

        let id = egui::Id::new("box selection");
        let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
        ctx.layer_painter(layer_id).rect(
            rect,
            0.0,
            self.selection_fill,
            (1.0, self.selection_outline),
        );

        self.interaction = if self.input.mouse[btn].released {
            Interaction::None
        } else {
            Interaction::BoxSelection { start }
        };
    }

    fn node_creation(&mut self, ctx: &egui::Context, position: egui::Pos2, first: bool) {
        let area_position = {
            let mut rect = ctx.input().screen_rect;
            rect.max.x = (rect.max.x - 250.0).max(rect.min.x);
            rect.max.y = (rect.max.y - 200.0).max(rect.min.y);
            let egui::Rect { min, max } = rect;
            position.clamp(min, max)
        };

        let area = egui::Area::new("NodeCreation").order(egui::Order::Foreground);
        let area = area.movable(false).current_pos(area_position);

        let egui::InnerResponse { response, inner } = area.show(ctx, |ui| {
            ui.set_width(250.0);

            let frame = egui::Frame::popup(ui.style());
            let out = frame.show(ui, |ui| {
                let response = ui.text_edit_singleline(&mut self.search);
                if first {
                    response.request_focus();
                }

                ui.add_space(4.0);

                let scroll = egui::ScrollArea::vertical();
                let scroll = scroll.auto_shrink([false; 2]);

                let out = scroll.show(ui, |ui| {
                    let align = egui::Align::Min;
                    let layout = egui::Layout::top_down(align).with_cross_justify(true);
                    let out = ui.with_layout(layout, |ui| {
                        for group in creator_menu() {
                            if !self.search.is_empty() && group.nothing(&self.search) {
                                continue;
                            }

                            let out = ui.horizontal_wrapped(|ui| {
                                for &(target, builder) in group.items {
                                    let label: Option<egui::WidgetText> = if self.search.is_empty()
                                    {
                                        Some(target.into())
                                    } else {
                                        sublime_fuzzy::best_match(&self.search, target)
                                            .map(|result| mark_matches(result, target).into())
                                    };

                                    if let Some(text) = label {
                                        if ui.button(text).clicked() {
                                            return Some(builder);
                                        }
                                    }
                                }

                                None
                            });

                            if let Some(builder) = out.inner {
                                return Some(builder);
                            }
                        }

                        None
                    });

                    out.inner
                });

                out.inner
            });
            out.inner
        });

        if let Some(builder) = inner {
            let node = builder(&mut self.storage);
            self.storage.nodes[node].position = position - self.pan_offset;
            self.interaction = Interaction::None;
        } else if response.clicked_elsewhere() {
            self.interaction = Interaction::None;
        } else {
            let first = false;
            self.interaction = Interaction::NodeCreation { position, first };
        }
    }
}

fn mark_matches(result: sublime_fuzzy::Match, target: &str) -> egui::text::LayoutJob {
    use egui::epaint::{
        text::{LayoutJob, TextFormat},
        Color32, FontFamily, FontId,
    };

    let mut last_end = 0;
    let mut job = LayoutJob::default();
    let base = TextFormat {
        font_id: FontId::new(14.0, FontFamily::Proportional),
        color: Color32::WHITE,
        ..Default::default()
    };

    let matched = TextFormat {
        font_id: FontId::new(14.0, FontFamily::Proportional),
        color: Color32::WHITE,
        background: Color32::DARK_RED,
        ..Default::default()
    };

    for c in result.continuous_matches() {
        let prefix = target
            .chars()
            .skip(last_end)
            .take(c.start() - last_end)
            .collect::<String>();

        let current = target
            .chars()
            .skip(c.start())
            .take(c.len())
            .collect::<String>();

        job.append(&prefix, 0.0, base.clone());
        job.append(&current, 0.0, matched.clone());

        last_end = c.start() + c.len();
    }

    if last_end != target.len() {
        let last = target.chars().skip(last_end).collect::<String>();
        job.append(&last, 0.0, base);
    }

    job
}

type BuilderFn = fn(&mut Storage) -> Node;

struct Group<'a> {
    items: &'a [(&'a str, BuilderFn)],
}

impl<'a> Group<'a> {
    fn nothing(&self, query: &str) -> bool {
        self.items
            .iter()
            .all(|(target, _)| sublime_fuzzy::best_match(query, target).is_none())
    }
}

fn creator_menu<'a>() -> &'a [Group<'a>] {
    use crate::nodes::math::{Binary, Math, Unary};
    use crate::nodes::{builtin, channel, input, logic, master, math, uv};
    use naga::{BinaryOperator, MathFunction, UnaryOperator};

    &[
        // input
        Group {
            items: &[
                ("Boolean", logic::InputBoolean::spawn),
                ("Color", input::Color::spawn),
                ("Slider", input::Slider::spawn),
                ("Constant", input::Constant::spawn),
            ],
        },
        Group {
            items: &[
                ("Float", input::InputFloat::spawn),
                ("Vector2", input::InputVector2::spawn),
                ("Vector3", input::InputVector3::spawn),
                ("Vector4", input::InputVector4::spawn),
            ],
        },
        // math ops
        Group {
            items: &[
                ("Negate", |s| Unary::spawn(s, UnaryOperator::Negate)),
                ("Not", |s| Unary::spawn(s, UnaryOperator::Not)),
                ("Add", |s| Binary::spawn(s, BinaryOperator::Add)),
                ("Subtract", |s| Binary::spawn(s, BinaryOperator::Subtract)),
                ("Multiply", |s| Binary::spawn(s, BinaryOperator::Multiply)),
                ("Divide", |s| Binary::spawn(s, BinaryOperator::Divide)),
                ("Modulo", |s| Binary::spawn(s, BinaryOperator::Modulo)),
            ],
        },
        // math comparison
        Group {
            items: &[
                ("Abs", |s| Math::spawn(s, MathFunction::Abs)),
                ("Min", |s| Math::spawn(s, MathFunction::Min)),
                ("Max", |s| Math::spawn(s, MathFunction::Max)),
                ("Clamp", |s| Math::spawn(s, MathFunction::Clamp)),
            ],
        },
        // math trigonometry
        Group {
            items: &[
                ("Cos", |s| Math::spawn(s, MathFunction::Cos)),
                ("Cosh", |s| Math::spawn(s, MathFunction::Cosh)),
                ("Acos", |s| Math::spawn(s, MathFunction::Acos)),
                ("Acosh", |s| Math::spawn(s, MathFunction::Acosh)),
            ],
        },
        Group {
            items: &[
                ("Sin", |s| Math::spawn(s, MathFunction::Sin)),
                ("Sinh", |s| Math::spawn(s, MathFunction::Sinh)),
                ("Asin", |s| Math::spawn(s, MathFunction::Asin)),
                ("Asinh", |s| Math::spawn(s, MathFunction::Asinh)),
            ],
        },
        Group {
            items: &[
                ("Tan", |s| Math::spawn(s, MathFunction::Tan)),
                ("Tanh", |s| Math::spawn(s, MathFunction::Tanh)),
                ("Atan", |s| Math::spawn(s, MathFunction::Atan)),
                ("Atanh", |s| Math::spawn(s, MathFunction::Atanh)),
                ("Atan2", |s| Math::spawn(s, MathFunction::Atan2)),
            ],
        },
        // math decomposition
        Group {
            items: &[
                ("Ceil", |s| Math::spawn(s, MathFunction::Ceil)),
                ("Floor", |s| Math::spawn(s, MathFunction::Floor)),
                ("Round", |s| Math::spawn(s, MathFunction::Round)),
                ("Fract", |s| Math::spawn(s, MathFunction::Fract)),
                ("Trunc", |s| Math::spawn(s, MathFunction::Trunc)),
                // TODO: Modf,Frexp,Ldexp,
            ],
        },
        // exponent
        Group {
            items: &[
                ("Exp", |s| Math::spawn(s, MathFunction::Exp)),
                ("Exp2", |s| Math::spawn(s, MathFunction::Exp2)),
                ("Log", |s| Math::spawn(s, MathFunction::Log)),
                ("Log2", |s| Math::spawn(s, MathFunction::Log2)),
                ("Pow", |s| Math::spawn(s, MathFunction::Pow)),
            ],
        },
        // ...

        // geometry
        Group {
            items: &[
                // TODO: Outer, FaceForward, Refract
                ("Dot", |s| Math::spawn(s, MathFunction::Dot)),
                ("Cross", |s| Math::spawn(s, MathFunction::Cross)),
                ("Distance", |s| Math::spawn(s, MathFunction::Distance)),
                ("Length", |s| Math::spawn(s, MathFunction::Length)),
                ("Normalize", |s| Math::spawn(s, MathFunction::Normalize)),
                ("Reflect", |s| Math::spawn(s, MathFunction::Reflect)),
            ],
        },
        // computational
        Group {
            items: &[
                // TODO: Fma, Mix
                ("Sign", |s| Math::spawn(s, MathFunction::Sign)),
                ("Step", |s| Math::spawn(s, MathFunction::Step)),
                ("SmoothStep", |s| Math::spawn(s, MathFunction::SmoothStep)),
                ("Sqrt", |s| Math::spawn(s, MathFunction::Sqrt)),
                ("Inverse Sqrt", |s| {
                    Math::spawn(s, MathFunction::InverseSqrt)
                }),
            ],
        },
        // ...
        Group {
            items: &[
                ("Posterize", math::Posterize::spawn),
                ("Derivative", math::Derivative::spawn),
                ("Remap", math::Remap::spawn),
                ("Select", logic::Select::spawn),
                ("Comparison", logic::Comparison::spawn),
            ],
        },
        // channel
        Group {
            items: &[
                ("Combine", channel::Combine::spawn),
                ("Split", channel::Split::spawn),
                ("Swizzle", channel::Swizzle::spawn),
            ],
        },
        // builtin
        Group {
            items: &[
                ("Blackbody", builtin::Blackbody::spawn),
                ("GradientNoise", builtin::GradientNoise::spawn),
                ("SimpleNoise", builtin::SimpleNoise::spawn),
            ],
        },
        // uv
        Group {
            items: &[("Flipbook", uv::Flipbook::spawn)],
        },
        //master
        Group {
            items: &[
                ("FragmentInputs", master::FragmentInputs::spawn),
                ("Master", master::Master::spawn),
                ("Triangle", master::Triangle::spawn),
                ("Fullscreen", master::Fullscreen::spawn),
            ],
        },
    ]
}
