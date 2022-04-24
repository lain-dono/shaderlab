use super::util::*;
use crate::builder::expr::*;
use crate::builder::*;
use crate::workspace::{Data, Fragment, Node, Port, PreviewBuilder, Storage};
use naga::{BinaryOperator, DerivativeAxis, Expression, MathFunction, UnaryOperator};

#[allow(clippy::type_complexity)]
fn math_kind<'a>(fun: MathFunction) -> Option<(&'a [(&'a str, Data)], &'a str, Data)> {
    macro_rules! def {
        (match $fun_var:ident { $(
            $fun:ident
            ( $( $arg_name:literal : $arg_ty:ident ),+ )
            -> $r_name:literal : $r_ty:ident,
        )+ } ) => {
            match $fun_var {
                $(
                    MathFunction::$fun => (
                        &[$( ($arg_name, Data::$arg_ty), )+],
                        $r_name, Data::$r_ty
                    ),
                )+

                _ => return None,
            }
        };

    }

    Some(def! (
        match fun {
            // comparison
            Abs("in": FloatOrVector) -> "out": FloatOrVector,
            Min("a": FloatOrVector, "b": FloatOrVector) -> "out": FloatOrVector,
            Max("a": FloatOrVector, "b": FloatOrVector) -> "out": FloatOrVector,
            Clamp("in": FloatOrVector, "min": FloatOrVector, "max": FloatOrVector) -> "out": FloatOrVector,

            // trigonometry
            Cos("in": FloatOrVector) -> "out": FloatOrVector,
            Sin("in": FloatOrVector) -> "out": FloatOrVector,
            Tan("in": FloatOrVector) -> "out": FloatOrVector,

            Cosh("in": FloatOrVector) -> "out": FloatOrVector,
            Sinh("in": FloatOrVector) -> "out": FloatOrVector,
            Tanh("in": FloatOrVector) -> "out": FloatOrVector,

            Acos("in": FloatOrVector) -> "out": FloatOrVector,
            Asin("in": FloatOrVector) -> "out": FloatOrVector,
            Atan("in": FloatOrVector) -> "out": FloatOrVector,

            Acosh("in": FloatOrVector) -> "out": FloatOrVector,
            Asinh("in": FloatOrVector) -> "out": FloatOrVector,
            Atanh("in": FloatOrVector) -> "out": FloatOrVector,

            Atan2("a": FloatOrVector, "b": FloatOrVector) -> "out": FloatOrVector,

            // decomposition
            Ceil("in": FloatOrVector) -> "out": FloatOrVector,
            Floor("in": FloatOrVector) -> "out": FloatOrVector,
            Round("in": FloatOrVector) -> "out": FloatOrVector,

            Fract("in": FloatOrVector) -> "out": FloatOrVector,
            Trunc("in": FloatOrVector) -> "out": FloatOrVector,
            // Modf,
            // Frexp,
            // Ldexp,

            // exponent
            Exp("in": FloatOrVector) -> "out": FloatOrVector,
            Exp2("in": FloatOrVector) -> "out": FloatOrVector,
            Log("in": FloatOrVector) -> "out": FloatOrVector,
            Log2("in": FloatOrVector) -> "out": FloatOrVector,
            Pow("a": FloatOrVector, "b": FloatOrVector) -> "out": FloatOrVector,

            // geometry
            Dot("a": VectorAny, "b": VectorAny) -> "out": Float,
            //Outer,
            Cross("a": Vector3, "b": Vector3) -> "out": Float,
            Distance("a": Vector3, "b": Vector3) -> "out": Float,
            Length("in": VectorAny) -> "out": Float,
            Normalize("in": VectorAny) -> "out": FloatOrVector,
            //FaceForward,
            Reflect("in": VectorAny, "normal": VectorAny) -> "out": VectorAny,
            //Refract,

            // computational
            Sign("in": FloatOrVector) -> "out": FloatOrVector,
            //Fma,
            //Mix,
            Step("edge": FloatOrVector, "in": FloatOrVector) -> "out": FloatOrVector,
            SmoothStep("edge1": FloatOrVector, "edge2": FloatOrVector, "in": FloatOrVector) -> "out": FloatOrVector,
            Sqrt("in": FloatOrVector) -> "out": FloatOrVector,
            InverseSqrt("in": FloatOrVector) -> "out": FloatOrVector,
            /*
            Inverse,
            Transpose,
            Determinant,
            // bits
            CountOneBits,
            ReverseBits,
            ExtractBits,
            InsertBits,
            // data packing
            Pack4x8snorm,
            Pack4x8unorm,
            Pack2x16snorm,
            Pack2x16unorm,
            Pack2x16float,
            // data unpacking
            Unpack4x8snorm,
            Unpack4x8unorm,
            Unpack2x16snorm,
            Unpack2x16unorm,
            Unpack2x16float,
            */
        }
    ))
}

pub struct Math {
    fun: MathFunction,
    result: Port,
    arg: Port,
    arg1: Option<Port>,
    arg2: Option<Port>,
    arg3: Option<Port>,
}

impl Math {
    pub fn spawn(storage: &mut Storage, fun: MathFunction) -> Node {
        storage.spawn(format!("{:?}", fun), 100.0, |ctx, node| {
            let (args, out_label, out_data) = math_kind(fun).unwrap();

            Self {
                fun,
                result: ctx.output(node, out_label, Fragment, out_data, None),
                arg: ctx.input(node, args[0].0, Fragment, args[0].1, None),
                arg1: args
                    .get(1)
                    .map(|&(label, data)| ctx.input(node, label, Fragment, data, None)),
                arg2: args
                    .get(2)
                    .map(|&(label, data)| ctx.input(node, label, Fragment, data, None)),
                arg3: args
                    .get(3)
                    .map(|&(label, data)| ctx.input(node, label, Fragment, data, None)),
            }
        })
    }
}

impl PreviewBuilder for Math {
    fn output_expr(&self, node: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);
        let expr = Expression::Math {
            fun: self.fun,
            arg: function.for_input(self.arg)?,
            arg1: self.arg1.map(|arg| function.for_input(arg)).transpose()?,
            arg2: self.arg2.map(|arg| function.for_input(arg)).transpose()?,
            arg3: self.arg3.map(|arg| function.for_input(arg)).transpose()?,
        };

        let expr = function.emit(expr);

        function.named_expr(node, expr);

        Ok(expr)
    }
}

pub struct Unary {
    op: UnaryOperator,
    expr: Port,
    result: Port,
}

impl Unary {
    pub fn spawn(storage: &mut Storage, op: UnaryOperator) -> Node {
        storage.spawn(format!("{:?}", op), 100.0, |ctx, node| {
            let data = match op {
                UnaryOperator::Negate => Data::FloatOrVector,
                UnaryOperator::Not => Data::Boolean,
            };
            let result = ctx.output(node, "out", Fragment, data, None);
            let expr = ctx.input(node, "a", Fragment, data, None);
            Self { op, expr, result }
        })
    }
}

impl PreviewBuilder for Unary {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);
        let expr = function.for_input(self.expr)?;
        let op = self.op;
        Ok(function.emit(Expression::Unary { op, expr }))
    }

    fn show_preview(&self) -> bool {
        matches!(self.op, UnaryOperator::Negate)
    }
}

pub struct Binary {
    op: BinaryOperator,
    left: Port,
    right: Port,
    result: Port,
}

impl Binary {
    pub fn spawn(storage: &mut Storage, op: BinaryOperator) -> Node {
        storage.spawn(format!("{:?}", op), 100.0, |ctx, node| Self {
            op,
            result: ctx.output(node, "out", Fragment, Data::FloatOrVector, None),
            left: ctx.input(node, "a", Fragment, Data::FloatOrVector, None),
            right: ctx.input(node, "b", Fragment, Data::FloatOrVector, None),
        })
    }
}

impl PreviewBuilder for Binary {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);
        let (_, left, right) = resolve_pair_min(function, self.left, self.right)?;
        let op = self.op;
        Ok(function.emit(Expression::Binary { left, op, right }))
    }
}

pub struct Derivative {
    axis: DerivativeAxis,
    input: Port,
    result: Port,
}

impl Derivative {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Derivative", 100.0, |ctx, node| Self {
            axis: DerivativeAxis::Width,
            result: ctx.output(node, "out", Fragment, Data::FloatOrVector, None),
            input: ctx.input(node, "in", Fragment, Data::FloatOrVector, None),
        })
    }
}

impl PreviewBuilder for Derivative {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.axis, naga::DerivativeAxis::X, "X");
            ui.selectable_value(&mut self.axis, naga::DerivativeAxis::Y, "Y");
            ui.selectable_value(&mut self.axis, naga::DerivativeAxis::Width, "Width");
        });
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);

        let expr = function.for_input(self.input)?;
        let axis = self.axis;

        Ok(function.emit(Expression::Derivative { axis, expr }))
    }
}

pub struct Posterize {
    input: Port,
    steps: Port,
    result: Port,
}

impl Posterize {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Posterize", 100.0, |ctx, node| Self {
            result: ctx.output(node, "out", Fragment, Data::FloatOrVector, None),
            input: ctx.input(node, "in", Fragment, Data::FloatOrVector, None),
            steps: ctx.input(node, "steps", Fragment, Data::FloatOrVector, None),
        })
    }
}

impl PreviewBuilder for Posterize {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);

        let (dst, input, steps) = resolve_pair_min(function, self.input, self.steps)?;

        let one = dst.splat(function, Float(1.0))?;
        let inv_steps = (Wrap(one) / Wrap(steps)).emit(function)?;
        let div_input = (Wrap(input) / inv_steps).emit(function)?;

        let floor_input = function.emit(Expression::Math {
            fun: MathFunction::Floor,
            arg: div_input,
            arg1: None,
            arg2: None,
            arg3: None,
        });

        (Wrap(floor_input) * inv_steps).emit(function)
    }
}

pub struct Remap {
    input: Port,
    in_range: Port,
    out_range: Port,
    result: Port,
}

impl Remap {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Remap", 100.0, |ctx, node| Self {
            result: ctx.output(node, "out", Fragment, Data::FloatOrVector, None),
            input: ctx.input(node, "in", Fragment, Data::FloatOrVector, None),
            in_range: ctx.input(node, "in_range", Fragment, Data::Vector2, None),
            out_range: ctx.input(node, "out_range", Fragment, Data::Vector2, None),
        })
    }
}

impl PreviewBuilder for Remap {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);

        let input = function.for_input(self.input)?;
        let in_range = function.for_input_resolve(self.in_range, VectorKind::V2)?;
        let out_range = function.for_input_resolve(self.out_range, VectorKind::V2)?;

        let src = VectorKind::parse(function.extract_type(input)?).unwrap();

        let (in_x, in_y) = (AccessIndex(in_range, 0), AccessIndex(in_range, 1));
        let (out_x, out_y) = (AccessIndex(out_range, 0), AccessIndex(out_range, 1));

        let in_x = Wrap(src.splat(function, in_x)?);
        let in_y = Wrap(src.splat(function, in_y)?);
        let out_x = Wrap(src.splat(function, out_x)?);
        let out_y = Wrap(src.splat(function, out_y)?);

        (out_x + (Wrap(input) - in_x) * (out_y - out_x) / (in_y - in_x)).emit(function)
    }
}
