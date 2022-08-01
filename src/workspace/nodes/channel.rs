use super::super::builder::{expr::*, *};
use crate::workspace::{Data, Fragment, Node, Port, PreviewBuilder, Storage};

pub struct Combine {
    r: Port,
    g: Port,
    b: Port,
    a: Port,
    rgba: Port,
    rgb: Port,
    rg: Port,
}

impl Combine {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Combine", 100.0, |ctx, node| Self {
            r: ctx.input(node, "r", Fragment, Data::Float, None),
            g: ctx.input(node, "g", Fragment, Data::Float, None),
            b: ctx.input(node, "b", Fragment, Data::Float, None),
            a: ctx.input(node, "a", Fragment, Data::Float, None),
            rgba: ctx.output(node, "rgba", Fragment, Data::Vector4, None),
            rgb: ctx.output(node, "rgb", Fragment, Data::Vector3, None),
            rg: ctx.output(node, "rg", Fragment, Data::Vector2, None),
        })
    }
}

impl PreviewBuilder for Combine {
    fn output_expr(&self, _node: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        let r = function.for_input_float(self.r)?;
        let g = function.for_input_float(self.g)?;
        if output == self.rg {
            return [r, g].emit(function);
        }

        let b = function.for_input_float(self.b)?;
        if output == self.rgb {
            return [r, g, b].emit(function);
        }

        let a = function.for_input_float(self.a)?;
        if output == self.rgba {
            return [r, g, b, a].emit(function);
        }

        Err(EmitError::PortNotFound)
    }
}

pub struct Split {
    port: Port,
    r: Port,
    g: Port,
    b: Port,
    a: Port,
}

impl Split {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Split", 100.0, |ctx, node| Self {
            port: ctx.input(node, "in", Fragment, Data::VectorAny, None),
            r: ctx.output(node, "r", Fragment, Data::Float, None),
            g: ctx.output(node, "g", Fragment, Data::Float, None),
            b: ctx.output(node, "b", Fragment, Data::Float, None),
            a: ctx.output(node, "a", Fragment, Data::Float, None),
        })
    }
}

impl PreviewBuilder for Split {
    fn show_preview(&self) -> bool {
        false
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        let input = function.for_input_vector4(self.port)?;

        let mut rgba = [self.r, self.g, self.b, self.a].into_iter();
        let index = rgba.position(|p| p == output);
        let index = index.ok_or(EmitError::PortNotFound)? as u32;

        Ok(function.emit(naga::Expression::AccessIndex { base: input, index }))
    }
}

pub struct Swizzle {
    input: Port,
    output: Port,

    x: naga::SwizzleComponent,
    y: naga::SwizzleComponent,
    z: naga::SwizzleComponent,
    w: naga::SwizzleComponent,
}

impl Swizzle {
    pub fn spawn(storage: &mut Storage) -> Node {
        storage.spawn("Swizzle", 160.0, |ctx, node| Self {
            input: ctx.input(node, "in", Fragment, Data::VectorAny, None),
            output: ctx.output(node, "out", Fragment, Data::VectorAny, None),
            x: naga::SwizzleComponent::X,
            y: naga::SwizzleComponent::Y,
            z: naga::SwizzleComponent::Z,
            w: naga::SwizzleComponent::W,
        })
    }
}

impl PreviewBuilder for Swizzle {
    fn ui(&mut self, ui: &mut egui::Ui) {
        fn sel(ui: &mut egui::Ui, id: &str, value: &mut naga::SwizzleComponent) {
            egui::ComboBox::from_id_source(ui.id().with(id))
                .selected_text(format!("{:?}", value))
                .width(15.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(value, naga::SwizzleComponent::X, "X");
                    ui.selectable_value(value, naga::SwizzleComponent::Y, "Y");
                    ui.selectable_value(value, naga::SwizzleComponent::Z, "Z");
                    ui.selectable_value(value, naga::SwizzleComponent::W, "W");
                });
        }

        ui.horizontal(|ui| {
            sel(ui, "_x", &mut self.x);
            sel(ui, "_y", &mut self.y);
            sel(ui, "_z", &mut self.z);
            sel(ui, "_w", &mut self.w);
        });
    }

    fn output_expr(&self, _: Node, function: &mut FnBuilder, output: Port) -> EmitResult {
        assert_eq!(self.output, output);

        let vector = function.for_input(self.input)?;
        let size = match function.extract_type(vector)? {
            naga::TypeInner::Vector { size, .. } => *size,
            _ => unreachable!(),
        };

        Ok(function.emit(naga::Expression::Swizzle {
            vector,
            size,
            pattern: [self.x, self.y, self.z, self.w],
        }))
    }
}
