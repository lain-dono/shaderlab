use super::util::*;
use crate::builder::expr::*;
use crate::builder::*;
use crate::workspace::{Data, Fragment, Node, Port, PreviewBuilder, Storage};
use naga::{BinaryOperator, Expression};

pub struct InputBoolean {
    value: bool,
    output: Port,
}

impl InputBoolean {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Boolean", 80.0, |ctx, node| Self {
            value: false,
            output: ctx.output(node, "out", Fragment, Data::Boolean, None),
        })
    }
}

impl PreviewBuilder for InputBoolean {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.value, "value");
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);
        Bool(self.value).emit(function)
    }

    fn show_preview(&self) -> bool {
        false
    }
}

pub struct Comparison {
    op: BinaryOperator,
    left: Port,
    right: Port,
    result: Port,
}

impl Comparison {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Comparison", 100.0, |ctx, node| Self {
            op: BinaryOperator::Equal,
            result: ctx.output(node, "out", Fragment, Data::Boolean, None),
            left: ctx.input(node, "a", Fragment, Data::FloatOrVector, None),
            right: ctx.input(node, "b", Fragment, Data::FloatOrVector, None),
        })
    }
}

impl PreviewBuilder for Comparison {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered_justified(|ui| {
            ui.columns(3, |ui| {
                ui[0].selectable_value(&mut self.op, naga::BinaryOperator::LessEqual, "<=");
                ui[1].selectable_value(&mut self.op, naga::BinaryOperator::Equal, "==");
                ui[2].selectable_value(&mut self.op, naga::BinaryOperator::GreaterEqual, ">=");
            });
            ui.columns(3, |ui| {
                ui[0].selectable_value(&mut self.op, naga::BinaryOperator::Less, "<");
                ui[1].selectable_value(&mut self.op, naga::BinaryOperator::NotEqual, "!=");
                ui[2].selectable_value(&mut self.op, naga::BinaryOperator::Greater, ">");
            });
        });
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);
        let (_, left, right) = resolve_pair_min(function, self.left, self.right)?;
        let op = self.op;
        Ok(function.emit(Expression::Binary { left, op, right }))
    }

    fn show_preview(&self) -> bool {
        false
    }
}

pub struct Select {
    condition: Port,
    accept: Port,
    reject: Port,
    result: Port,
}

impl Select {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Select", 120.0, |ctx, node| Self {
            result: ctx.output(node, "out", Fragment, Data::FloatOrVector, None),
            condition: ctx.input(node, "cond", Fragment, Data::Boolean, None),
            accept: ctx.input(node, "accept", Fragment, Data::FloatOrVector, None),
            reject: ctx.input(node, "reject", Fragment, Data::FloatOrVector, None),
        })
    }
}

impl PreviewBuilder for Select {
    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.result, output);

        let condition = function.for_input(self.condition)?;
        let (_, accept, reject) = resolve_pair_min(function, self.accept, self.reject)?;

        Ok(function.emit(Expression::Select {
            condition,
            accept,
            reject,
        }))
    }
}
