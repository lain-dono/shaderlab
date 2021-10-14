pub mod input;
pub mod math;

use crate::controls::NodeId;

pub use naga::{ScalarKind as Scalar, VectorSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(Scalar),
    Vector(Scalar, VectorSize),
    Matrix(Scalar, VectorSize, VectorSize),
}

impl Type {
    pub const V2F: Self = Self::Vector(Scalar::Float, VectorSize::Bi);
    pub const V3F: Self = Self::Vector(Scalar::Float, VectorSize::Tri);
    pub const V4F: Self = Self::Vector(Scalar::Float, VectorSize::Quad);
}

pub type Message = Box<dyn DynMessage>;

pub trait DynMessage: downcast_rs::Downcast + std::fmt::Debug + Send + Sync {
    fn box_clone(&self) -> Box<dyn DynMessage>;
}

downcast_rs::impl_downcast!(DynMessage);

impl<T: std::any::Any + std::fmt::Debug + Clone + Send + Sync> DynMessage for T {
    fn box_clone(&self) -> Box<dyn DynMessage> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DynMessage> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

pub fn downcast_message<T: DynMessage + Clone>(message: Message) -> Option<T> {
    message.downcast_ref().cloned()
}

pub fn upcast_message(message: impl DynMessage + Clone) -> Message {
    Box::new(message)
}

#[derive(Debug)]
pub struct GenError;

pub trait Node: std::fmt::Debug + 'static {
    fn label(&self) -> &str;

    fn width(&self) -> u16 {
        150
    }

    fn inputs(&self) -> &[(&str, Type)];
    fn outputs(&self) -> &[(&str, Type)];

    fn update(&mut self, _node: NodeId, _message: Message) {}

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError>;

    fn view(
        &mut self,
        _node: NodeId,
    ) -> iced_native::Element<Box<dyn DynMessage>, iced_wgpu::Renderer> {
        iced_native::Space::new(iced_native::Length::Shrink, iced_native::Length::Shrink).into()
    }
}

pub fn node_return(inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
    if let ([Some(a)], []) = (inputs, outputs) {
        Ok(format!("return {};", a))
    } else {
        Err(GenError)
    }
}

#[derive(Clone, Default, Debug)]
pub struct Return;

impl Node for Return {
    fn label(&self) -> &str {
        "return"
    }

    fn width(&self) -> u16 {
        80
    }

    fn inputs(&self) -> &[(&str, Type)] {
        &[("color", Type::V4F)]
    }
    fn outputs(&self) -> &[(&str, Type)] {
        &[]
    }

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
        if let ([Some(a)], []) = (inputs, outputs) {
            Ok(format!("return {};", a))
        } else {
            Err(GenError)
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct NodeDebug;

impl Node for NodeDebug {
    fn label(&self) -> &str {
        "debug"
    }

    fn inputs(&self) -> &[(&str, Type)] {
        &[
            ("A_in", Type::V4F),
            ("B_in", Type::V4F),
            ("C_in", Type::V4F),
        ]
    }
    fn outputs(&self) -> &[(&str, Type)] {
        &[
            ("A_out", Type::V4F),
            ("B_out", Type::V4F),
            ("C_out", Type::V4F),
        ]
    }

    fn generate(&self, inputs: &[Option<String>], outputs: &[String]) -> Result<String, GenError> {
        let inputs = inputs
            .iter()
            .map(|i| match i {
                Some(s) => s.clone(),
                None => "None".to_string(),
            })
            .collect::<Vec<_>>();

        Ok(format!(
            "let ({}) = dbg({});",
            outputs.join(", "),
            inputs.join(", "),
        ))
    }
}
