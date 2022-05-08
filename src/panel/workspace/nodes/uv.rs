use crate::builder::expr::*;
use crate::builder::*;
use crate::workspace::{Data, Fragment, Node, Port, PreviewBuilder, Storage};
use naga::{Expression, Handle, MathFunction};

pub struct Flipbook {
    output: Port,

    uv: Port,
    width: Port,
    height: Port,
    tile: Port,

    flip_x: bool,
    flip_y: bool,
}

impl Flipbook {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Flipbook", 100.0, |ctx, node| Self {
            output: ctx.output(node, "out", Fragment, Data::Vector2, None),

            uv: ctx.output(node, "uv", Fragment, Data::Vector2, None),
            width: ctx.output(node, "width", Fragment, Data::Float, None),
            height: ctx.output(node, "height", Fragment, Data::Float, None),
            tile: ctx.output(node, "tile", Fragment, Data::Float, None),

            flip_x: false,
            flip_y: false,
        })
    }
}

impl PreviewBuilder for Flipbook {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.flip_x, "invert x");
        ui.checkbox(&mut self.flip_y, "invert y");
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);

        let flip_x = Float(if self.flip_x { 1.0 } else { 0.0 });
        let flip_y = Float(if self.flip_y { 1.0 } else { 0.0 });

        let uv = Wrap(function.for_input_vector2(self.uv)?);
        let tile = Wrap(function.for_input_float(self.tile)?);
        let width = Wrap(function.for_input_float(self.width)?);
        let height = Wrap(function.for_input_float(self.height)?);

        let tile = call_math_2(MathFunction::Modf, tile.0, (width * height).emit(function)?);

        let one = Float(1.0);

        let tile_count = Wrap([one; 2].emit(function)?) / Wrap([width, height].emit(function)?);

        let tile_count_x = AccessIndex(tile_count.emit(function)?, 0);

        let floor_tile_count = (tile.clone() * tile_count_x).emit(function)?;
        let floor_tile_count = call_math_1(MathFunction::Floor, floor_tile_count);

        let tile_y = flip_y * height - (floor_tile_count.clone() + flip_y);
        let tile_y = call_math_1(MathFunction::Abs, tile_y.emit(function)?);

        let tile_x = flip_x * width - ((tile - width * floor_tile_count) + flip_x);
        let tile_x = call_math_1(MathFunction::Abs, tile_x.emit(function)?);

        let out = (uv + Wrap([tile_x, tile_y].emit(function)?)) * tile_count;

        out.emit(function)
    }
}

fn call_math_1(fun: MathFunction, arg0: Handle<Expression>) -> Expr {
    call_math(fun, arg0, None, None, None)
}

fn call_math_2(fun: MathFunction, arg0: Handle<Expression>, arg1: Handle<Expression>) -> Expr {
    call_math(fun, arg0, arg1, None, None)
}

fn call_math_3(
    fun: MathFunction,
    arg0: Handle<Expression>,
    arg1: Handle<Expression>,
    arg2: Handle<Expression>,
) -> Expr {
    call_math(fun, arg0, arg1, arg2, None)
}

fn call_math(
    fun: MathFunction,
    arg0: Handle<Expression>,
    arg1: impl Into<Option<Handle<Expression>>>,
    arg2: impl Into<Option<Handle<Expression>>>,
    arg3: impl Into<Option<Handle<Expression>>>,
) -> Expr {
    Expr(Expression::Math {
        fun,
        arg: arg0,
        arg1: arg1.into(),
        arg2: arg2.into(),
        arg3: arg3.into(),
    })
}
