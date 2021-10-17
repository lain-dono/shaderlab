use crate::{builder::NodeBuilder, controls::edge::Edge};
pub use naga::{ScalarKind as Scalar, VectorSize};

pub mod input;
pub mod master;
pub mod math;

slotmap::new_key_type! { pub struct NodeId; }

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

pub trait BoxedNode {
    fn boxed() -> Box<dyn Node>;
}

impl<T: Node + Default> BoxedNode for T {
    fn boxed() -> Box<dyn Node> {
        Box::new(Self::default())
    }
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

pub struct NodeDescriptor<'a> {
    pub label: &'a str,
    pub width: u16,
    pub inputs: &'a [(&'a str, Type)],
    pub outputs: &'a [(&'a str, Type)],
}

pub trait Node: std::fmt::Debug + 'static + downcast_rs::Downcast + NodeBuilder {
    fn desc(&self) -> NodeDescriptor<'_>;

    fn add_edge(&mut self, edge: Edge, is_input: bool) {
        log::info!(
            "node add edge {} -> {} {}",
            edge.output(),
            edge.input(),
            is_input
        );
    }

    fn remove_edge(&mut self, edge: Edge, is_input: bool) {
        log::info!(
            "node remove edge {} -> {} {}",
            edge.output(),
            edge.input(),
            is_input
        );
    }

    fn update(&mut self, _node: NodeId, _message: Message) {}

    fn view(
        &mut self,
        _node: NodeId,
    ) -> iced_native::Element<Box<dyn DynMessage>, iced_wgpu::Renderer> {
        iced_native::Space::new(iced_native::Length::Shrink, iced_native::Length::Shrink).into()
    }
}

downcast_rs::impl_downcast!(Node);
