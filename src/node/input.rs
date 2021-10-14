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

use super::{GenError, Message, Node, NodeId};
use iced_native::{
    widget::{slider, text_input},
    Column, Element, Length, Slider,
};
use iced_wgpu::Renderer;

#[derive(Clone, Debug, Default)]
pub struct Input {
    data: [(text_input::State, f32); 4],
}

impl Input {
    pub fn boxed() -> Box<dyn Node> {
        Box::new(Self::default())
    }
}

impl Node for Input {
    fn label(&self) -> &str {
        "v4f"
    }

    fn width(&self) -> u16 {
        100
    }

    fn inputs(&self) -> &[(&str, super::Type)] {
        &[]
    }

    fn outputs(&self) -> &[(&str, super::Type)] {
        &[("out", super::Type::V4F)]
    }

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
        if let ([], [ret]) = (inputs, outputs) {
            Ok(format!(
                "let {} = vec4<f32>({:?}, {:?}, {:?}, {:?});",
                ret, self.data[0].1, self.data[1].1, self.data[2].1, self.data[3].1
            ))
        } else {
            Err(GenError)
        }
    }

    fn update(&mut self, _node: NodeId, message: Message) {
        let (index, value) = super::downcast_message::<(usize, f32)>(message).unwrap();
        self.data[index].1 = value;
    }

    fn view(&mut self, _node: NodeId) -> Element<Message, Renderer> {
        let col = Column::new().padding([2, 2]).spacing(2);

        self.data
            .iter_mut()
            .enumerate()
            .fold(col, |col, (i, (state, value))| {
                let value = format!("{:?}", *value);
                let input = text_input::TextInput::new(state, "", &value, move |s| {
                    let value: f32 = s.parse().unwrap_or(0.0);
                    super::upcast_message((i, value))
                })
                .padding(1)
                .width(Length::Fill)
                .style(crate::style::Node)
                .size(crate::style::FONT_SIZE);
                col.push(input)
            })
            .into()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Color {
    sliders: [slider::State; 4],
    color: iced_native::Color,
}

impl Color {
    pub fn boxed() -> Box<dyn Node> {
        Box::new(Self::default())
    }
}

impl Node for Color {
    fn label(&self) -> &str {
        "color"
    }

    fn width(&self) -> u16 {
        100
    }

    fn inputs(&self) -> &[(&str, super::Type)] {
        &[]
    }

    fn outputs(&self) -> &[(&str, super::Type)] {
        &[("out", super::Type::V4F)]
    }

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
        if let ([], [ret]) = (inputs, outputs) {
            Ok(format!(
                "let {} = vec4<f32>({:?}, {:?}, {:?}, {:?});",
                ret, self.color.r, self.color.g, self.color.b, self.color.a
            ))
        } else {
            Err(GenError)
        }
    }

    fn update(&mut self, _node: NodeId, message: Message) {
        self.color = super::downcast_message::<iced_native::Color>(message).unwrap();
    }

    fn view(&mut self, _node: NodeId) -> Element<Message, Renderer> {
        rgba_sliders(&mut self.sliders, self.color).map(super::upcast_message)
    }
}

#[allow(dead_code)]
fn rgba_sliders(
    sliders: &mut [slider::State; 4],
    color: iced_native::Color,
) -> Element<iced_native::Color, Renderer> {
    use iced_native::Color;

    let [r, g, b, a] = sliders;

    let r = Slider::new(r, 0.0..=1.0, color.r, move |r| Color { r, ..color }).step(0.01);
    let g = Slider::new(g, 0.0..=1.0, color.g, move |g| Color { g, ..color }).step(0.01);
    let b = Slider::new(b, 0.0..=1.0, color.b, move |b| Color { b, ..color }).step(0.01);
    let a = Slider::new(a, 0.0..=1.0, color.a, move |a| Color { a, ..color }).step(0.01);

    Element::from(Column::new().push(r).push(g).push(b).push(a))
}

#[derive(Clone, Debug, Default)]
pub struct Position {}

impl Position {
    pub fn boxed() -> Box<dyn Node> {
        Box::new(Self::default())
    }
}

impl Node for Position {
    fn label(&self) -> &str {
        "position"
    }

    fn width(&self) -> u16 {
        100
    }

    fn inputs(&self) -> &[(&str, super::Type)] {
        &[]
    }

    fn outputs(&self) -> &[(&str, super::Type)] {
        &[("out", super::Type::V4F)]
    }

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
        if let ([], [ret]) = (inputs, outputs) {
            Ok(format!("let {} = position;", ret,))
        } else {
            Err(GenError)
        }
    }
}
