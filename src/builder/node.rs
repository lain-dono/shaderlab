use crate::builder::FunctionBuilder;
use naga::{Arena, Expression, Handle, Span};

pub mod input;
pub mod master;
pub mod math;

pub trait NodeBuilder {
    fn expr(
        &self,
        nodes: &NodeArena,
        _function: &mut FunctionBuilder<'_>,
        _output: usize,
    ) -> Option<Handle<Expression>>;
}

pub trait AnyNode: downcast_rs::Downcast + NodeBuilder {}
downcast_rs::impl_downcast!(AnyNode);

impl<T: downcast_rs::Downcast + NodeBuilder> AnyNode for T {}

pub type Port = (Handle<Node>, usize);

pub type Node = Box<dyn AnyNode>;

#[derive(Default)]
pub struct NodeArena {
    pub inner: Arena<Node>,
}

impl NodeArena {
    pub fn push(&mut self, node: impl AnyNode) -> Handle<Node> {
        self.inner.append(Box::new(node), Span::default())
    }

    pub fn expr(
        &self,
        function: &mut FunctionBuilder<'_>,
        (node, output): Port,
    ) -> Option<Handle<Expression>> {
        self.inner[node].expr(self, function, output)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PortType {
    Boolean,

    Vector1,
    Vector2,
    Vector3,
    Vector4,

    Matrix2,
    Matrix3,
    Matrix4,

    Dynamic,
    DynamicVector,
    DynamicMatrix,

    Texture2D,
    Texture2DArray,
    Texture3D,
    Cubemap,
    //Gradient,
    //SamplerState,
}

impl PortType {
    pub fn can_place(self, ty: Self) -> bool {
        matches!(
            (self, ty),
            (
                Self::Dynamic | Self::DynamicVector,
                Self::Vector1 | Self::Vector2 | Self::Vector3 | Self::Vector4,
            ) | (
                Self::Dynamic | Self::DynamicMatrix,
                Self::Matrix2 | Self::Matrix3 | Self::Matrix4,
            ) | (Self::Boolean, Self::Boolean)
                | (Self::Vector1, Self::Vector1)
                | (Self::Vector2, Self::Vector2)
                | (Self::Vector3, Self::Vector3)
                | (Self::Vector4, Self::Vector4)
                | (Self::Matrix2, Self::Matrix2)
                | (Self::Matrix3, Self::Matrix3)
                | (Self::Matrix4, Self::Matrix4)
        )
    }
}
