use iced_wgpu::Renderer;
use iced_winit::{
    alignment, button, scrollable, slider, {Alignment, Command, Element, Program},
    {Button, Column, Container, Row, Rule, Scrollable, Slider, Text},
    {Color, Length, Point, Rectangle},
};
use slotmap::SlotMap;

pub mod edge;
pub mod node;
pub mod port;
pub mod swizzle;
pub mod workspace;

pub use self::{
    edge::{Connection, Edge, Pending, PortId},
    node::{NodeId, NodeWidget},
    port::Port,
    workspace::Workspace,
};

fn fix_name(s: &str) -> String {
    s.chars()
        .map(|c| if !c.is_ascii_alphanumeric() { '_' } else { c })
        .collect()
}

#[derive(Debug, Clone)]
pub enum Message {
    Background(Color),
    Check(bool),
    StartBezier(Pending),
    CancelBezier,

    NodeInternal(NodeId, Box<dyn crate::node::DynMessage>),

    AddNode(fn() -> Box<dyn crate::node::Node>),
    RemoveNode(NodeId),
    StartDrag(NodeId),
    EndDrag(NodeId),
    Move(Point),

    Fix(NodeId),

    ScrollToTop,
    ScrollToBottom,
    Scrolled(f32),

    Save,
    Todo,

    SwizzleSelect(swizzle::Message),
}

#[derive(Default)]
pub struct Controls {
    source: String,
    background_color: Color,
    sliders: [slider::State; 3],

    nodes: SlotMap<NodeId, NodeWidget>,
    edges: Vec<Edge>,
    workspace: workspace::State,
    drag: Option<NodeId>,
    drag_last: Point,

    scrollable: scrollable::State,
    _scroll_to_top: button::State,
    _scroll_to_bottom: button::State,

    save: button::State,

    swizzle: swizzle::State,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            source: "    return vec4<f32>(0.02, 0.02, 0.02, 1.0);\n".to_string(),
            background_color: Color::BLACK,
            nodes: {
                let dbg = Box::new(crate::node::NodeDebug);
                let add = crate::node::math::Add::boxed();
                let ret = Box::new(crate::node::Return);

                let mut nodes = SlotMap::with_key();
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(50.0, 100.0), dbg));
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(300.0, 100.0), add));
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(500.0, 100.0), ret));
                nodes
            },
            edges: vec![],
            drag: None,
            drag_last: Point::ORIGIN,
            ..Default::default()
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn workspace(&self) -> Rectangle {
        self.workspace.bounds()
    }
    pub fn source(&self) -> &str {
        &self.source
    }

    fn check_add(&self, edge: Edge) -> bool {
        let mut graph = crate::graph::Graph::default();
        for edge in &self.edges {
            graph.add_edge(edge.output.node, edge.input.node);
        }
        graph.add_edge(edge.output.node, edge.input.node);
        !graph.is_reachable(edge.input.node, edge.output.node)
    }

    pub fn fix_node_position(&mut self, node: NodeId) {
        let base_offset = Point::ORIGIN - self.workspace.bounds().position();

        let node = if let Some(node) = self.nodes.get(node) {
            node
        } else {
            log::warn!("can't find node: {:?}", node);
            return;
        };

        for (port, input) in node.inputs.iter().enumerate() {
            let position = input.slot();
            for edge in &mut self.edges {
                if edge.input.node == node.id && edge.input.port == PortId(port) {
                    edge.input.position = position + base_offset;
                }
            }
        }

        for (port, output) in node.outputs.iter().enumerate() {
            let position = output.slot();
            for edge in &mut self.edges {
                if edge.output.node == node.id && edge.output.port == PortId(port) {
                    edge.output.position = position + base_offset;
                }
            }
        }

        self.workspace.request_redraw();
    }

    fn try_traverse(&mut self) {
        use colored::Colorize as _;

        log::info!("try_traverse");

        let start = self.nodes.values().find_map(|node| {
            if node.label() == "return" {
                Some(node.id)
            } else {
                None
            }
        });

        struct Label {
            node: (NodeId, String),
            port: (PortId, String),
        }

        impl ToString for Label {
            fn to_string(&self) -> String {
                format!(
                    "{}_{}_{}_{}",
                    fix_name(&self.node.1),
                    self.port.1,
                    self.node.0.to_string(),
                    self.port.0 .0,
                )
            }
        }

        if let Some(start) = start {
            let mut graph = crate::graph::Graph::default();
            for edge in &self.edges {
                graph.add_edge(edge.input.node, edge.output.node);
            }

            let mut code = Vec::new();

            let result = graph.dfs(start, |id| -> Result<(), crate::node::GenError> {
                let node = &self.nodes[id];

                let inputs: Vec<_> = node
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(input_port, _)| {
                        let output = self.edges.iter().find_map(|e| {
                            if e.input.node == node.id && e.input.port == PortId(input_port) {
                                Some(e.output)
                            } else {
                                None
                            }
                        });

                        output.map(|output| {
                            let node = &self.nodes[output.node];
                            Label {
                                node: (node.id, node.label().to_string()),
                                port: (output.port, node.outputs[output.port.0].label.clone()),
                            }
                            .to_string()
                        })
                    })
                    .collect();

                let outputs: Vec<_> = node
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(port, output)| {
                        Label {
                            node: (node.id, node.label().to_string()),
                            port: (PortId(port), output.label.clone()),
                        }
                        .to_string()
                    })
                    .collect();

                let deps: std::collections::HashSet<NodeId> = self
                    .edges
                    .iter()
                    .filter(|edge| edge.input.node == id)
                    .map(|edge| edge.output.node)
                    .collect();

                node.node
                    .generate(&inputs, &outputs)
                    .map(|gen| code.push((node.id, gen, deps)))
            });

            match result {
                Ok(()) => {
                    code.reverse();

                    // dfs sucks
                    code.sort_by(|(a, _, a_deps), (b, _, b_deps)| {
                        if a_deps.contains(b) {
                            std::cmp::Ordering::Greater
                        } else if b_deps.contains(a) {
                            std::cmp::Ordering::Less
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });

                    let mut source = String::new();

                    for (_node, op, _) in &code {
                        use std::fmt::Write;
                        writeln!(&mut source, "  {}", op).unwrap();
                    }
                    //println!("{{");
                    //println!("{}", source);
                    //println!("}}");
                    self.source = source;
                }
                Err(err) => {
                    log::warn!("wtf: {}", format!("{:?}", err).red())
                }
            }
        } else {
            log::warn!("no start node");
        }
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        fn next_frame(message: Message) -> Command<Message> {
            Command::perform(iced_futures::futures::future::ready(message), |f| f)
        }

        let base_offset = Point::ORIGIN - self.workspace.bounds().position();
        match message {
            Message::Save => {
                let shader = crate::scene::Scene::wrap(&self.source);
                std::thread::spawn(|| {
                    use native_dialog::FileDialog;
                    let path = FileDialog::new()
                        .set_filename("result.wgsl")
                        //.set_location("~/Desktop")
                        //.add_filter("PNG Image", &["png"])
                        //.add_filter("JPEG Image", &["jpg", "jpeg"])
                        .show_save_single_file()
                        .unwrap();

                    /*
                    let path = match path {
                        Some(path) => path,
                        None => return,
                    };

                    let yes = MessageDialog::new()
                        .set_type(MessageType::Info)
                        .set_title("Do you want to open the file?")
                        .set_text(&format!("{:#?}", path))
                        .show_confirm()
                        .unwrap();

                    std::
                    */
                    if let Some(path) = path {
                        let _ = dbg!(std::fs::write(path, shader));
                    }
                });
            }
            Message::Fix(node) => self.fix_node_position(node),

            Message::NodeInternal(node, message) => {
                self.nodes[node].node.update(node, message);

                self.try_traverse();
            }

            Message::AddNode(node) => {
                let node = node();
                log::info!("add node {:?}", node);

                let bounds = self.workspace.bounds();
                let position = Point::ORIGIN + (bounds.center() - bounds.position());

                self.nodes
                    .insert_with_key(|id| NodeWidget::new(id, position, node));
            }

            Message::RemoveNode(node) => {
                self.nodes.remove(node);
                self.edges
                    .retain(|edge| edge.input.node != node && edge.output.node != node);

                self.workspace.request_redraw();
            }
            Message::Todo => (),
            Message::ScrollToTop => {
                self.scrollable.snap_to(0.0);
                //.latest_offset = 0.0;
            }
            Message::ScrollToBottom => {
                self.scrollable.snap_to(1.0);
                //variant.latest_offset = 1.0;
            }
            Message::Scrolled(_offset) => {
                //self.scrollable.latest_offset = offset;
            }

            Message::StartDrag(node) => {
                log::info!("start drag");
                self.drag = Some(node);
                self.fix_node_position(node);
            }
            Message::Move(position) => {
                let delta = position - self.drag_last;
                self.drag_last = position;
                if let Some(id) = self.drag {
                    let node = &mut self.nodes[id];
                    node.position = node.position + delta;

                    self.fix_node_position(id);
                    return next_frame(Message::Fix(id));
                }
            }
            Message::EndDrag(node) => {
                log::info!("end drag");
                self.drag = None;
                self.fix_node_position(node);

                return next_frame(Message::Fix(node));
            }

            //Message::Move(po)
            Message::Background(color) => self.background_color = color,
            Message::StartBezier(from) => {
                let from = from.translate(base_offset);

                if let Some(to) = self.workspace.pending() {
                    if let Some(edge) = Edge::new(from, to).filter(Edge::not_same_node) {
                        if let Some(index) = self.edges.iter().position(|e| e.eq_node_port(&edge)) {
                            log::info!("remove edge {} -> {}", edge.output, edge.input);
                            self.edges.remove(index);
                        // only one edge for input port
                        // and not cycle
                        } else if self
                            .edges
                            .iter()
                            .all(|e| !e.input.eq_node_port(&edge.input))
                            && self.check_add(edge)
                        {
                            log::info!("create edge {} -> {}", edge.output, edge.input);

                            self.edges.push(edge);
                        }
                    }

                    self.workspace.end();
                    self.workspace.request_redraw();

                    self.try_traverse();
                } else {
                    log::info!("start {}", from);
                    self.workspace.start(from);
                }
            }
            Message::CancelBezier => {
                log::info!("cancel edge creation");
                self.workspace.end();
                self.workspace.request_redraw();
                self.drag = None;
            }
            Message::Check(_) => (),

            Message::SwizzleSelect(message) => self.swizzle.update(message),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let sliders = rgb_sliders(&mut self.sliders, self.background_color);

        let _sidebar = Column::new()
            .padding(10)
            .spacing(10)
            .push(Text::new("Background color").color(Color::WHITE))
            .push(sliders)
            .push(
                Text::new(format!("{:#?}", self.background_color))
                    .size(14)
                    .color(Color::WHITE),
            );

        let content = self.nodes.values_mut().fold(
            Workspace::new(&mut self.workspace, &self.edges, Message::CancelBezier),
            |content, state| content.push(state.position, state.widget()),
        );

        /*
        let scroll_to_bottom = Text::new("Scroll to bottom");
        let scroll_to_bottom = Button::new(&mut self.scroll_to_bottom, scroll_to_bottom)
            .width(Length::Fill)
            .padding(2)
            .on_press(Message::ScrollToBottom);

        let scroll_to_top = Text::new("Scroll to top");
        let scroll_to_top = Button::new(&mut self.scroll_to_top, scroll_to_top)
            .width(Length::Fill)
            .padding(2)
            .on_press(Message::ScrollToTop);
            */

        let scrollable = Scrollable::new(&mut self.scrollable)
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(Message::Scrolled)
            .scrollbar_width(0)
            .scroller_width(0);

        let sidebar = node_list(scrollable);

        /*
        let sidebar = sidebar
            .push(scroll_to_bottom);
            .push(sidebar)
            .push(Text::new("The End."))
            .push(scroll_to_top);
            */

        let sidebar = Column::new()
            .push(sidebar)
            .push(self.swizzle.view().map(Message::SwizzleSelect))
            .push(Button::new(&mut self.save, Text::new("save")).on_press(Message::Save));

        let sidebar = Container::new(sidebar)
            .style(crate::style::Sidebar)
            .width(Length::Units(200))
            .height(Length::Fill);

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Start)
            .push(sidebar)
            .push(content)
            .into()
    }
}

fn rgb_sliders(sliders: &mut [slider::State; 3], color: Color) -> Element<Message, Renderer> {
    let [r, g, b] = sliders;

    let r = Slider::new(r, 0.0..=1.0, color.r, move |r| Color { r, ..color }).step(0.01);
    let g = Slider::new(g, 0.0..=1.0, color.g, move |g| Color { g, ..color }).step(0.01);
    let b = Slider::new(b, 0.0..=1.0, color.b, move |b| Color { b, ..color }).step(0.01);

    Element::from(Column::new().push(r).push(g).push(b)).map(Message::Background)
}

#[allow(dead_code)]
fn rgba_sliders(sliders: &mut [slider::State; 4], color: Color) -> Element<Message, Renderer> {
    let [r, g, b, a] = sliders;

    let r = Slider::new(r, 0.0..=1.0, color.r, move |r| Color { r, ..color }).step(0.01);
    let g = Slider::new(g, 0.0..=1.0, color.g, move |g| Color { g, ..color }).step(0.01);
    let b = Slider::new(b, 0.0..=1.0, color.b, move |b| Color { b, ..color }).step(0.01);
    let a = Slider::new(a, 0.0..=1.0, color.a, move |a| Color { a, ..color }).step(0.01);

    Element::from(Column::new().push(r).push(g).push(b).push(a)).map(Message::Background)
}

fn node_list(scrollable: Scrollable<Message, Renderer>) -> Scrollable<Message, Renderer> {
    type Desc<'a> = [(&'a str, fn() -> Box<dyn crate::node::Node>)];

    trait ExtScrollable: Sized {
        fn header(self, label: &str) -> Self;
        fn subheader(self, label: &str) -> Self;
        fn item(self, label: &str) -> Self;
        fn divider(self) -> Self;

        fn row(self, labels: &[&str]) -> Self;
        fn rowx(self, items: &Desc) -> Self;
    }

    impl<'a> ExtScrollable for Scrollable<'a, Message, Renderer> {
        fn header(self, label: &str) -> Self {
            let text = Text::new(label)
                .size(20)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center);
            self.push(text).divider()
        }

        fn subheader(self, label: &str) -> Self {
            let text = Text::new(label)
                .size(16)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left);
            let text = Container::new(text).padding([0, 0, 0, 16]);
            self.push(text)
        }

        fn item(self, label: &str) -> Self {
            let text = Text::new(label)
                .size(14)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left);
            let text = Container::new(text).padding([0, 0, 0, 16]);
            self.push(text)
        }

        fn divider(self) -> Self {
            self.push(Rule::horizontal(8))
        }

        fn row(self, labels: &[&str]) -> Self {
            use crate::widget::press_pad::PressPad;
            let row = Row::new()
                .width(Length::Fill)
                .padding([0, 0, 0, 16])
                .spacing(8);
            let row = labels.iter().fold(row, |row, &label| {
                let text = Text::new(label)
                    .size(14)
                    .horizontal_alignment(alignment::Horizontal::Left);
                row.push(PressPad::new(
                    text,
                    Message::AddNode(crate::node::math::Abs::boxed),
                ))
            });

            self.push(row)
        }

        fn rowx(self, items: &Desc) -> Self {
            use crate::widget::press_pad::PressPad;
            let row = Row::new()
                .width(Length::Fill)
                .padding([0, 0, 0, 32])
                .spacing(8);
            let row = items.iter().cloned().fold(row, |row, (label, create)| {
                let text = Text::new(label)
                    .size(14)
                    .horizontal_alignment(alignment::Horizontal::Left);
                row.push(PressPad::new(text, Message::AddNode(create)))
            });

            self.push(row)
        }
    }

    scrollable
        /*
        .header("Artistic")
        .subheader("Adjustment")
        .item("Channel Mixer")
        .item("Contrast")
        .item("Hue")
        .item("Invert Colors")
        .item("Replace Color")
        .item("Saturation")
        .item("White Balance")
        .subheader("Blend")
        .item("Blend")
        .subheader("Filter")
        .item("Dither")
        .subheader("Mask")
        .item("Channel Mask")
        .item("Color Mask")
        .subheader("Normal")
        .item("Normal Blend")
        .item("Normal From Height")
        .item("Normal Strength")
        .item("Normal Unpack")
        .subheader("Utility")
        .item("Colorspace Conversion")
        //
        .header("Channel")
        .item("Combine")
        .item("Flip")
        .item("Split")
        .item("Swizzle")
        //
        */
        .header("Input")
        .subheader("Basic")
        //.row(&["bool", "color", "f32"])
        //.row(&["vec2", "vec3", "vec4"])
        .rowx(&[("vec4", crate::node::input::Input::boxed)])
        .rowx(&[("color", crate::node::input::Color::boxed)])
        .rowx(&[("position", crate::node::input::Position::boxed)])
        //.item("Boolean")
        //.item("Color")
        //.item("Constant")
        //.item("Integer")
        //.item("Slider")
        //.item("Time")
        //.item("Vector 1")
        //.item("Vector 2")
        //.item("Vector 3")
        //.item("Vector 4")
        /*
                .subheader("Geometry")
                .item("Bitangent Vector")
                .item("Normal Vector")
                .item("Position")
                .item("Screen Position")
                .item("Tangent Vector")
                .item("UV")
                .item("Vertex Color")
                .item("View Direction")
                .subheader("Gradient")
                .item("Gradient")
                .item("Sample Gradient")
                .subheader("Matrix")
                .item("Matrix 2x2")
                .item("Matrix 3x3")
                .item("Matrix 4x4")
                .item("Transformation Matrix")
                .subheader("PBR")
                .item("Dielectric Specular")
                .item("Metal Reflectance")
                .subheader("Scene")
                .item("Ambient")
                .item("Camera")
                .item("Fog")
                .item("Object")
                .item("Reflection Probe")
                .item("Scene Color")
                .item("Scene Depth")
                .item("Screen")
                .subheader("Texture")
                .item("Cubemap Asset")
                .item("Sample Cubemap")
                .item("Sample Texture 2D")
                .item("Sample Texture 2D Array")
                .item("Sample Texture 2D LOD")
                .item("Sample Texture 3D")
                .item("Sampler State")
                .item("Texel Size")
                .item("Texture 2D Array Asset")
                .item("Texture 2D Asset")
                .item("Texture 3D Asset")
                //
                .header("Master")
                .item("PBR Master")
                .item("Unlit Master")
                //
                */
        /*
        Cos Cosh Acos
        Sin Sinh Asin
        Tan Tanh Atan Atan2
         */
        .header("Math")
        .subheader("Basic")
        .rowx(&[
            ("Add", crate::node::math::Add::boxed),
            ("Sub", crate::node::math::Sub::boxed),
            ("Mul", crate::node::math::Mul::boxed),
            ("Div", crate::node::math::Div::boxed),
            ("Rem", crate::node::math::Rem::boxed),
        ])
        .rowx(&[
            ("Abs", crate::node::math::Abs::boxed),
            ("Exp", crate::node::math::Exp::boxed),
            ("Exp2", crate::node::math::Exp2::boxed),
            ("Log", crate::node::math::Log::boxed),
            ("Sqrt", crate::node::math::Sqrt::boxed),
        ])
        .rowx(&[
            ("Length", crate::node::math::Length::boxed),
            ("Normalize", crate::node::math::Normalize::boxed),
            ("Pow", crate::node::math::Pow::boxed),
        ])
        .subheader("Trigonometry")
        .rowx(&[
            ("Cos", crate::node::math::Cos::boxed),
            ("Cosh", crate::node::math::Cosh::boxed),
            ("Acos", crate::node::math::Acos::boxed),
        ])
        .rowx(&[
            ("Sin", crate::node::math::Sin::boxed),
            ("Sinh", crate::node::math::Sinh::boxed),
            ("Asin", crate::node::math::Asin::boxed),
        ])
        .rowx(&[
            ("Tan", crate::node::math::Tan::boxed),
            ("Tanh", crate::node::math::Tanh::boxed),
            ("Atan", crate::node::math::Atan::boxed),
            ("Atan2", crate::node::math::Atan2::boxed),
        ])
        .rowx(&[
            ("To Radians", crate::node::math::DegresToRadians::boxed),
            ("To Degrees", crate::node::math::DegresToRadians::boxed),
        ])
        .subheader("Derivative")
        .rowx(&[
            ("dpdx", crate::node::math::DpDx::boxed),
            ("dpdy", crate::node::math::DpDy::boxed),
        ])
        .subheader("Rounding")
        .rowx(&[
            ("Ceil", crate::node::math::Ceil::boxed),
            ("Floor", crate::node::math::Floor::boxed),
            ("Round", crate::node::math::Round::boxed),
            ("Sign", crate::node::math::Sign::boxed),
        ])
        .rowx(&[
            ("Trunc", crate::node::math::Trunc::boxed),
            ("Step", crate::node::math::Step::boxed),
        ])
    //
    /*
    .subheader("Advanced")
    //.item("Absolute")
    //.item("Exponential")
    //.item("Length")
    //.item("Log")
    //.item("Modulo")
    .item("Negate")
    //.item("Normalize")
    .item("Posterize")
    .item("Reciprocal")
    .item("Reciprocal Square Root")
    .subheader("Derivative")
    //.item("DDX")
    //.item("DDXY")
    //.item("DDY")
    .subheader("Interpolation")
    .item("Inverse Lerp")
    .item("Lerp")
    .item("Smoothstep")
    .subheader("Matrix")
    .item("Matrix Construction")
    .item("Matrix Determinant")
    .item("Matrix Split")
    .item("Matrix Transpose")
    .subheader("Range")
    .item("Clamp")
    .item("Fraction")
    .item("Maximum")
    .item("Minimum")
    .item("One Minus")
    .item("Random Range")
    .item("Remap")
    .item("Saturate")
    //.subheader("Round")
    //.item("Ceiling")
    //.item("Floor")
    //.item("Round")
    //.item("Sign")
    //.item("Step")
    //.item("Truncate")
    .subheader("Vector")
    .item("Cross Product")
    .item("Distance")
    .item("Dot Product")
    .item("Fresnel Effect")
    .item("Projection")
    .item("Reflection")
    .item("Rejection")
    .item("Rotate About Axis")
    .item("Sphere Mask")
    .item("Transform")
    .subheader("Wave")
    .item("Noise Sine Wave")
    .item("Sawtooth Wave")
    .item("Square Wave")
    .item("Triangle Wave")
    //
    .header("Procedural")
    .item("Checkerboard")
    .subheader("Noise")
    .item("Gradient Noise")
    .item("Simple Noise")
    .item("Voronoi")
    .subheader("Shapes")
    .item("Ellipse")
    .item("Polygon")
    .item("Rectangle")
    .item("Rounded Rectangle")
    //
    .header("Utility")
    .item("Custom Function")
    .item("Preview")
    .item("Sub Graph")
    .subheader("Logic")
    .item("All")
    .item("And")
    .item("Any")
    .item("Branch")
    .item("Comparison")
    .item("Is Front Face")
    .item("Is Infinite")
    .item("Is NaN")
    .item("Nand")
    .item("Not")
    .item("Or")
    //
    .header("UV")
    .item("Flipbook")
    .item("Polar Coordinates")
    .item("Radial Shear")
    .item("Rotate")
    .item("Spherize")
    .item("Tiling And Offset")
    .item("Triplanar")
    .item("Twirl")
    */
}
