use crate::builder::{expr::Emit, FunctionBuilder, NodeBuilder};
use crate::node::{port, Event, Node, NodeElement, NodeId, NodeWidget, Output, PortId, Type};
use crate::style;
use crate::widget::slider;
use iced_graphics::{Column, Container};
use iced_native::Element;
use iced_wgpu::Renderer;
use naga::{Expression, Handle};

pub type Float = f64;
pub type Vector1 = [f64; 1];
pub type Vector2 = [f64; 2];
pub type Vector3 = [f64; 3];
pub type Vector4 = [f64; 4];

impl NodeBuilder for Float {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port == PortId(0)).then(|| self.emit(function))
    }
}

impl NodeBuilder for Vector1 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port == PortId(0)).then(|| self.emit(function))
    }
}

impl NodeBuilder for Vector2 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port == PortId(0)).then(|| self.emit(function))
    }
}

impl NodeBuilder for Vector3 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port == PortId(0)).then(|| self.emit(function))
    }
}

impl NodeBuilder for Vector4 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port == PortId(0)).then(|| self.emit(function))
    }
}

pub struct Triangle(NodeWidget);

impl Default for Triangle {
    fn default() -> Self {
        Self(NodeWidget::new(
            "triangle",
            75,
            &[],
            &[("out", super::Type::Vector4)],
        ))
    }
}

impl Node for Triangle {
    fn inputs(&self) -> &[port::State] {
        &self.0.ports.inputs
    }
    fn outputs(&self) -> &[port::State] {
        &self.0.ports.outputs
    }
    fn update(&mut self, event: Event) {
        self.0.update(event);
    }

    fn view(&mut self, node: NodeId) -> NodeElement {
        self.0.view(node, None)
    }

    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port == PortId(0) {
            use crate::builder::expr::*;

            let vertex_index = FunctionArgument(0).emit(function);
            let x = sub(vertex_index, 1u64);
            let y = sub(mul(and(vertex_index, 1u64), 2u64), 1u64);

            Some(
                [
                    Let::new("x", float(sint(x))).emit(function),
                    Let::new("y", float(sint(y))).emit(function),
                    0f64.emit(function),
                    1f64.emit(function),
                ]
                .emit(function),
            )
        } else {
            None
        }
    }
}

pub struct Fullscreen(NodeWidget);

impl Default for Fullscreen {
    fn default() -> Self {
        Self(NodeWidget::new(
            "fullscreen",
            75,
            &[],
            &[("out", super::Type::Vector4)],
        ))
    }
}

impl Node for Fullscreen {
    fn inputs(&self) -> &[port::State] {
        &self.0.ports.inputs
    }
    fn outputs(&self) -> &[port::State] {
        &self.0.ports.outputs
    }

    fn update(&mut self, event: Event) {
        self.0.update(event);
    }

    fn view(&mut self, node: NodeId) -> NodeElement {
        self.0.view(node, None)
    }

    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port == PortId(0) {
            use crate::builder::expr::*;

            let vertex_index = FunctionArgument(0).emit(function);
            let u = and(shift_left(vertex_index, 1u64), 2u64);
            let v = and(vertex_index, 2u64);
            let u = add(mul(sint(u), 2i64), -1i64);
            let v = add(mul(sint(v), -2i64), 1i64);

            Some(
                [
                    Let::new("x", float(sint(u))).emit(function),
                    Let::new("y", float(sint(v))).emit(function),
                    0f64.emit(function),
                    1f64.emit(function),
                ]
                .emit(function),
            )
        } else {
            None
        }
    }
}

pub struct Input(NodeWidget);

impl Input {
    pub fn new(ty: Type) -> Self {
        let inputs = [
            ("x", Type::Vector1),
            ("y", Type::Vector1),
            ("z", Type::Vector1),
            ("w", Type::Vector1),
        ];
        let (label, inputs) = match ty {
            Type::Vector1 => ("float", &inputs[0..1]),
            Type::Vector2 => ("vector2", &inputs[0..2]),
            Type::Vector3 => ("vector3", &inputs[0..3]),
            Type::Vector4 => ("vector4", &inputs[0..4]),
        };
        Self(NodeWidget::new(label, 100, inputs, &[("out", ty)]))
    }
}

impl Node for Input {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        let vector = [
            self.0.defaults[0].value[0],
            self.0.defaults[1].value[0],
            self.0.defaults[2].value[0],
            self.0.defaults[3].value[0],
        ];
        vector.expr(function, output)
    }

    fn inputs(&self) -> &[port::State] {
        &self.0.ports.inputs
    }
    fn outputs(&self) -> &[port::State] {
        &self.0.ports.outputs
    }
    fn update(&mut self, event: Event) {
        self.0.update(event)
    }
    fn view(&mut self, node: NodeId) -> NodeElement {
        self.0.view(node, None)
    }
}

pub struct Color {
    base: NodeWidget,
    sliders: [slider::State; 4],
    color: iced_wgpu::wgpu::Color,
    vector: Vector4,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            base: NodeWidget::new("color", 100, &[], &[("out", super::Type::Vector4)]),
            sliders: Default::default(),
            color: iced_wgpu::wgpu::Color::default(),
            vector: Default::default(),
        }
    }
}

impl Node for Color {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        self.vector.expr(function, output)
    }

    fn inputs(&self) -> &[port::State] {
        &self.base.ports.inputs
    }
    fn outputs(&self) -> &[port::State] {
        &self.base.ports.outputs
    }

    fn update(&mut self, event: Event) {
        match event {
            Event::Dynamic(message) => {
                self.color = super::downcast_message::<iced_wgpu::wgpu::Color>(message).unwrap();
                self.vector = [self.color.r, self.color.g, self.color.b, self.color.a];
            }
            _ => self.base.update(event),
        }
    }

    fn view(&mut self, node: NodeId) -> NodeElement {
        let controls = Container::new(rgba_sliders(&mut self.sliders, self.color));
        self.base.view(node, Some(controls.into()))
    }
}

#[allow(dead_code)]
fn rgba_sliders(
    sliders: &mut [slider::State; 4],
    color: iced_wgpu::wgpu::Color,
) -> Element<Box<dyn super::DynMessage>, Renderer> {
    use iced_wgpu::wgpu::Color;

    let [r, g, b, a] = sliders;

    let r = style::Node::slider(r, 0.0..=1.0, color.r, move |r| Color { r, ..color }).step(0.01);
    let g = style::Node::slider(g, 0.0..=1.0, color.g, move |g| Color { g, ..color }).step(0.01);
    let b = style::Node::slider(b, 0.0..=1.0, color.b, move |b| Color { b, ..color }).step(0.01);
    let a = style::Node::slider(a, 0.0..=1.0, color.a, move |a| Color { a, ..color }).step(0.01);

    let col = Column::new().padding([2, 2]).spacing(2);

    Element::from(col.push(r).push(g).push(b).push(a)).map(super::upcast_message)
}
