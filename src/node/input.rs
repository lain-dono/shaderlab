pub mod basic {
    pub struct Boolean;
    pub struct Color;
    pub struct Constant;
    pub struct Scalar;
    pub struct Slider;
    pub struct Time;
    pub struct Vector1;
    pub struct Vector2;
    pub struct Vector3;
    pub struct Vector4;
}

pub mod geometry {
    pub struct BitangentVector;
    pub struct NormalVector;
    pub struct TangentVector;

    pub struct Position;
    pub struct ScreenPosition;

    pub struct UV;
    pub struct VertexColor;
    pub struct ViewDirection;
}

pub mod gradient {
    pub struct Gradient;
    pub struct SampleGradient;
}

pub mod matrix {
    pub struct Matrix2x2;
    pub struct Matrix3x3;
    pub struct Matrix4x4;
    pub struct TransformationMatrix;
}

pub mod pbr {
    pub struct DielectricSpecular;
    pub struct MetalReflectance;
}

pub mod scene {
    pub struct Ambient;
    pub struct Camera;
    pub struct Fog;
    pub struct BakedGI;
    pub struct Object;
    pub struct ReflectionProbe;
    pub struct SceneColor;
    pub struct SceneDepth;
    pub struct ScreenSize;
}

pub mod texture {
    pub struct CubemapAsset;
    pub struct SampleCubemap;
    pub struct SampleTexture2D;
    pub struct SampleTexture2DArray;
    pub struct SampleTexture2DLOD;
    pub struct SampleTexture3D;
    pub struct SamplerState;
    pub struct TexelSize;
    pub struct Texture2DArrayAsset;
    pub struct Texture2DAsset;
    pub struct Texture3DAsset;
}

use crate::builder::{expr::Emit, FunctionBuilder, NodeBuilder};
use crate::node::{Dynamic, Event, Node, NodeDescriptor, NodeId, Output, PortId};
use crate::style;
use crate::widget::slider;
use iced_native::{widget::text_input, Column, Container, Element, Length};
use iced_wgpu::Renderer;
use naga::{Expression, Handle};

#[derive(Debug, Default)]
pub struct Float {
    pub value: f64,
}

impl NodeBuilder for Float {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port() == PortId(0)).then(|| self.value.emit(function))
    }
}

#[derive(Debug, Default)]
pub struct Vec2 {
    pub value: [f64; 2],
}

impl NodeBuilder for Vec2 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port() == PortId(0)).then(|| self.value.emit(function))
    }
}

#[derive(Debug, Default)]
pub struct Vec3 {
    pub value: [f64; 3],
}

impl NodeBuilder for Vec3 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port() == PortId(0)).then(|| self.value.emit(function))
    }
}

#[derive(Debug, Default)]
pub struct Vec4 {
    pub value: [f64; 4],
}

impl NodeBuilder for Vec4 {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        (output.port() == PortId(0)).then(|| self.value.emit(function))
    }
}

#[derive(Default, Debug)]
pub struct Triangle;

impl NodeBuilder for Triangle {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port() == PortId(0) {
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

#[derive(Debug, Default)]
pub struct Fullscreen;

impl NodeBuilder for Fullscreen {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        if output.port() == PortId(0) {
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

#[derive(Debug, Default)]
pub struct Input {
    state: [text_input::State; 4],
    vector: Vec4,
}

impl NodeBuilder for Input {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        self.vector.expr(function, output)
    }
}

impl Node for Input {
    fn desc(&self) -> NodeDescriptor<'_> {
        NodeDescriptor {
            label: "vec4<f32>",
            width: 100,
            inputs: &[],
            outputs: &[("out", super::Type::V4F)],
        }
    }

    fn update(&mut self, event: Event) {
        if let Event::Dynamic(message) = event {
            let (index, value) = super::downcast_message::<(usize, f64)>(message).unwrap();
            self.vector.value[index] = value;
        }
    }

    fn view(&mut self, _node: NodeId) -> Element<Dynamic, Renderer> {
        let col = Column::new().padding([2, 2]).spacing(2);

        self.state
            .iter_mut()
            .zip(self.vector.value.iter())
            .enumerate()
            .fold(col, |col, (i, (state, value))| {
                let value = format!("{:?}", *value);
                let input = style::Node::input(state, "", &value, move |s| {
                    super::upcast_message((i, s.parse::<f64>().unwrap_or(0.0)))
                });
                col.push(input.width(Length::Fill))
            })
            .into()
    }
}

#[derive(Debug, Default)]
pub struct Color {
    sliders: [slider::State; 4],
    color: iced_wgpu::wgpu::Color,
    vector: Vec4,
}

impl NodeBuilder for Color {
    fn expr(&self, function: &mut FunctionBuilder, output: Output) -> Option<Handle<Expression>> {
        self.vector.expr(function, output)
    }
}

impl Node for Color {
    fn desc(&self) -> NodeDescriptor<'_> {
        NodeDescriptor {
            label: "color",
            width: 100,
            inputs: &[],
            outputs: &[("out", super::Type::V4F)],
        }
    }

    fn update(&mut self, event: Event) {
        if let Event::Dynamic(message) = event {
            self.color = super::downcast_message::<iced_wgpu::wgpu::Color>(message).unwrap();
            self.vector.value = [self.color.r, self.color.g, self.color.b, self.color.a];
        }
    }

    fn view(&mut self, _node: NodeId) -> Element<Dynamic, Renderer> {
        Container::new(rgba_sliders(&mut self.sliders, self.color).map(super::upcast_message))
            .into()
    }
}

#[allow(dead_code)]
fn rgba_sliders(
    sliders: &mut [slider::State; 4],
    color: iced_wgpu::wgpu::Color,
) -> Element<iced_wgpu::wgpu::Color, Renderer> {
    use iced_wgpu::wgpu::Color;

    let [r, g, b, a] = sliders;

    let r = style::Node::slider(r, 0.0..=1.0, color.r, move |r| Color { r, ..color }).step(0.01);
    let g = style::Node::slider(g, 0.0..=1.0, color.g, move |g| Color { g, ..color }).step(0.01);
    let b = style::Node::slider(b, 0.0..=1.0, color.b, move |b| Color { b, ..color }).step(0.01);
    let a = style::Node::slider(a, 0.0..=1.0, color.a, move |a| Color { a, ..color }).step(0.01);

    let col = Column::new().padding([2, 2]).spacing(2);

    Element::from(col.push(r).push(g).push(b).push(a))
}

#[derive(Clone, Debug, Default)]
pub struct Position;

impl NodeBuilder for Position {
    fn expr(&self, _function: &mut FunctionBuilder, _output: Output) -> Option<Handle<Expression>> {
        None
    }
}

impl Node for Position {
    fn desc(&self) -> NodeDescriptor<'_> {
        NodeDescriptor {
            label: "position",
            width: 100,
            inputs: &[],
            outputs: &[("out", super::Type::V4F)],
        }
    }
}

impl Node for Triangle {
    fn desc(&self) -> NodeDescriptor<'_> {
        NodeDescriptor {
            label: "triangle",
            width: 75,
            inputs: &[],
            outputs: &[("out", super::Type::V4F)],
        }
    }
}

impl Node for Fullscreen {
    fn desc(&self) -> NodeDescriptor<'_> {
        NodeDescriptor {
            label: "fullscreen",
            width: 75,
            inputs: &[],
            outputs: &[("out", super::Type::V4F)],
        }
    }
}
