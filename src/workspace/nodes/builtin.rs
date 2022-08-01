use super::super::builder::{expr::*, *};
use crate::workspace::{Data, Fragment, Node, Port, PreviewBuilder, Storage};

macro_rules! call_node {
    (
        $name:ident => $builtin:ident(
            $( $arg:ident : $arg_ty:ident ),+
        ) -> $ret_ty:ident

    ) => {

        pub struct $name {
            $($arg: Port,)+
            result: Port,
        }

        impl $name {
            pub fn spawn(storage: &mut Storage) -> Node {
                storage.spawn(stringify!($name), 150.0, |ctx, node| {
                    Self {
                        $( $arg: ctx.input(node, stringify!($arg), Fragment, Data::$arg_ty, None), )+
                        result: ctx.output(node, "out", Fragment, Data::$ret_ty, None),
                    }
                })
            }
        }

        impl PreviewBuilder for $name {
            fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
                assert_eq!(self.result, output);
                let args = [
                    $( call_node!(@ self $arg $arg_ty function)? ),+
                ];
                function.call(stringify!($builtin), args)
            }
        }

    };

    (@ $self:ident $arg:ident Float   $function:ident) => { $function.for_input_resolve($self.$arg, VectorKind::V1) };
    (@ $self:ident $arg:ident Vector2 $function:ident) => { $function.for_input_resolve($self.$arg, VectorKind::V2) };
    (@ $self:ident $arg:ident Vector3 $function:ident) => { $function.for_input_resolve($self.$arg, VectorKind::V3) };
    (@ $self:ident $arg:ident Vector4 $function:ident) => { $function.for_input_resolve($self.$arg, VectorKind::V4) };
    (@ $self:ident $arg:ident $_x:ident $function:ident) => { $function.for_input($self.$arg) };
}

call_node!(Blackbody => builtin_blackbody(k: Float) -> Vector3);
call_node!(SimpleNoise => builtin_simple_noise(uv: Vector2, scale: Float) -> Float);
call_node!(GradientNoise => builtin_gradient_noise(uv: Vector2, scale: Float) -> Float);
