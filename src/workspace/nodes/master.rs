use super::super::builder::{expr::*, *};
use crate::workspace::{Data, Fragment, Node, Port, PreviewBuilder, Storage, Vertex};

pub struct FragmentInputs {
    builtin_position: Port,    // vec4<f32>
    builtin_font_facing: Port, // bool

    position: Port,  // vec3
    normal: Port,    // vec3
    tangent: Port,   // vec3
    bitangent: Port, // vec3
    uv0: Port,       // vec2
    uv1: Port,       // vec2
}

impl FragmentInputs {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Fragment Inputs", 150.0, |ctx, node| Self {
            builtin_position: ctx.output(node, "frag position", Fragment, Data::Vector4, None),
            builtin_font_facing: ctx.output(
                node,
                "frag font facing",
                Fragment,
                Data::Boolean,
                None,
            ),
            position: ctx.output(node, "position", Fragment, Data::Vector3, None),
            normal: ctx.output(node, "normal", Fragment, Data::Vector3, None),
            tangent: ctx.output(node, "tangent", Fragment, Data::Vector3, None),
            bitangent: ctx.output(node, "bitangent", Fragment, Data::Vector3, None),
            uv0: ctx.output(node, "uv0", Fragment, Data::Vector2, None),
            uv1: ctx.output(node, "uv1", Fragment, Data::Vector2, None),
        })
    }
}

impl PreviewBuilder for FragmentInputs {
    fn show_preview(&self) -> bool {
        false
    }

    fn output_expr(&self, _node: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        let input = FunctionArgument(0).emit(function)?;
        match output {
            port if port == self.builtin_position => AccessIndex(input, 0).emit(function),
            port if port == self.builtin_font_facing => AccessIndex(input, 1).emit(function),

            port if port == self.position => todo!(),
            port if port == self.normal => todo!(),
            port if port == self.tangent => todo!(),
            port if port == self.bitangent => todo!(),
            port if port == self.uv0 => todo!(),
            port if port == self.uv1 => todo!(),

            _ => Err(EmitError::PortNotFound),
        }
    }
}

pub struct Master {
    position: Port,
    color: Port,
}

impl Master {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Master", 100.0, |ctx, node| Self {
            position: ctx.input(node, "position", Vertex, Data::Vector3, None),
            color: ctx.input(node, "color", Fragment, Data::Vector4, None),
        })
    }
}

impl PreviewBuilder for Master {
    fn output_expr(&self, _node: Node, _: &mut FnBuilder, _: Port) -> EmitResult {
        Err(EmitError::PortNotFound)
    }

    fn vertex(&self, _: Node, function: &mut FnBuilder) -> EmitResult {
        function.for_input_vector4(self.position)
    }

    fn fragment(&self, _: Node, function: &mut FnBuilder) -> EmitResult {
        function
            .for_input(self.color)
            .or_else(|_| expr_fill_white(function))
    }
}

pub struct Triangle {
    position: Port,
}

impl Triangle {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Triangle", 100.0, |ctx, node| Self {
            position: ctx.output(node, "position", Vertex, Data::Vector3, None),
        })
    }
}

impl PreviewBuilder for Triangle {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.position, output);
        expr_triangle(function)
    }

    fn vertex(&self, node: Node, function: &mut FnBuilder) -> EmitResult {
        self.output_expr(node, function, self.position)
    }

    fn fragment(&self, _: Node, function: &mut FnBuilder) -> EmitResult {
        expr_fill_white(function)
    }
}

pub struct Fullscreen {
    position: Port,
}

impl Fullscreen {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Fullscreen", 100.0, |ctx, node| Self {
            position: ctx.output(node, "position", Vertex, Data::Vector3, None),
        })
    }
}

impl PreviewBuilder for Fullscreen {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.position, output);
        expr_fullscreen(function)
    }

    fn vertex(&self, node: Node, function: &mut FnBuilder) -> EmitResult {
        self.output_expr(node, function, self.position)
    }

    fn fragment(&self, _: Node, function: &mut FnBuilder) -> EmitResult {
        expr_fill_white(function)
    }
}

pub fn expr_fill_white(function: &mut FnBuilder) -> EmitResult {
    [Float(1.0); 4].emit(function)
}

pub fn expr_fullscreen(function: &mut FnBuilder) -> EmitResult {
    let vertex_index = AccessIndex(FunctionArgument(0).emit(function)?, 0);

    let u = (vertex_index << Uint(1)) & Uint(2);
    let v = vertex_index & Uint(2);
    let u = (u.sint() * Sint(2) + Sint(-1)).float();
    let v = (v.sint() * Sint(-2) + Sint(1)).float();
    let u = Let::new("u", u).emit(function)?;
    let v = Let::new("v", v).emit(function)?;

    let zero = Float(0.0).emit(function)?;
    let one = Float(1.0).emit(function)?;

    [u, v, zero, one].emit(function)
}

pub fn expr_triangle(function: &mut FnBuilder) -> EmitResult {
    let vertex_index = AccessIndex(FunctionArgument(0).emit(function)?, 0);

    let u = vertex_index - Uint(1);
    let v = (vertex_index & Uint(1)) * Uint(2) - Uint(1);
    let u = Let::new("x", u.sint().float()).emit(function)?;
    let v = Let::new("y", v.sint().float()).emit(function)?;

    [u, v, Float(0.0).emit(function)?, Float(1.0).emit(function)?].emit(function)
}
