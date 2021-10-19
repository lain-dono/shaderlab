use iced_wgpu::Renderer;
use iced_winit::{
    alignment, button, scrollable, Alignment, Button, Column, Command, Container, Element, Length,
    Point, Program, Rectangle, Row, Rule, Scrollable, Text,
};

pub mod swizzle;
pub mod workspace;

use self::workspace::Workspace;
use crate::node::{BoxedNode as _, Message as NodeMessage, Node, NodeId, NodeMap, NodeWidget};

#[derive(Debug, Clone)]
pub enum Message {
    AddNode(fn() -> Box<dyn Node>),
    Workspace(NodeMessage),

    Fix(NodeId),

    ScrollToTop,
    ScrollToBottom,
    Scrolled(f32),

    Save,
    SwizzleSelect(swizzle::Message),
}

#[derive(Default)]
pub struct Controls {
    source: String,

    workspace: workspace::State,
    scrollable: scrollable::State,
    save: button::State,
    swizzle: swizzle::State,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            workspace: workspace::State::with_nodes({
                let color_a = crate::node::input::Color::boxed();
                let color_b = crate::node::input::Color::boxed();
                let triangle = crate::node::input::Triangle::boxed();
                let add = crate::node::math::Add::boxed();
                let master = crate::node::master::Master::boxed();

                let mut nodes = NodeMap::default();
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(50.0, 100.0), color_a));
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(50.0, 300.0), color_b));
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(300.0, 100.0), triangle));
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(300.0, 300.0), add));
                nodes.insert_with_key(|id| NodeWidget::new(id, Point::new(500.0, 100.0), master));
                nodes
            }),
            ..Default::default()
        }
    }

    pub fn workspace(&self) -> Rectangle {
        self.workspace.bounds()
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        fn next_frame(message: Message) -> Command<Message> {
            Command::perform(iced_futures::futures::future::ready(message), |f| f)
        }

        match message {
            Message::Save => {
                let shader = self.source.clone();
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
            Message::Fix(node) => self.workspace.fix_node_position(node),

            Message::AddNode(node) => self.workspace.add_node(node()),
            Message::Workspace(message) => match message {
                NodeMessage::Remove(node) => self.workspace.remove_node(node),
                NodeMessage::Dynamic(node, message) => {
                    self.workspace.update_node(node, message);
                    if let Some(source) = self.workspace.try_traverse() {
                        self.source = source;
                    }
                }
                NodeMessage::DragStart(node) => {
                    log::info!("start drag");
                    self.workspace.drag = Some(node);
                    self.workspace.fix_node_position(node);
                }
                NodeMessage::DragMove(position) => {
                    let delta = position - self.workspace.drag_last;
                    self.workspace.drag_last = position;
                    if let Some(id) = self.workspace.drag {
                        self.workspace.move_node(id, delta);
                        return next_frame(Message::Fix(id));
                    }
                }
                NodeMessage::DragEnd(node) => {
                    log::info!("end drag");
                    self.workspace.drag = None;
                    self.workspace.fix_node_position(node);
                    return next_frame(Message::Fix(node));
                }
                NodeMessage::StartEdge(from) => {
                    self.workspace.start_edge(from);
                    if let Some(source) = self.workspace.try_traverse() {
                        self.source = source;
                    }
                }
                NodeMessage::CancelEdge => {
                    log::info!("cancel edge creation");
                    self.workspace.end();
                    self.workspace.request_redraw();
                    self.workspace.drag = None;
                }
            },

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

            Message::SwizzleSelect(message) => self.swizzle.update(message),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let content = Workspace::new(&mut self.workspace);
        let content: Element<_, _> = content.into();
        let content = content.map(Message::Workspace);

        let scrollable = Scrollable::new(&mut self.scrollable)
            .padding(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(Message::Scrolled)
            .scrollbar_width(0)
            .scroller_width(0);

        let sidebar = node_list(scrollable);

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

/*
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
*/

fn node_list(scrollable: Scrollable<Message, Renderer>) -> Scrollable<Message, Renderer> {
    type Desc<'a> = [(&'a str, fn() -> Box<dyn Node>)];

    trait ExtScrollable: Sized {
        fn header(self, label: &str) -> Self;
        fn subheader(self, label: &str) -> Self;
        fn item(self, label: &str) -> Self;
        fn divider(self) -> Self;

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
        .rowx(&[("triangle", crate::node::input::Triangle::boxed)])
        .rowx(&[("fullscreen", crate::node::input::Fullscreen::boxed)])
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
