use crate::builder::{
    node::{NodeArena, NodeBuilder, Port},
    FunctionBuilder, ModuleBuilder, F32_4, U32,
};
use naga::{
    Binding, BuiltIn, EntryPoint, Expression, Handle, Interpolation, Sampling, ShaderStage,
    Statement,
};

pub struct Master {
    pub position: Option<Port>,
    pub color: Option<Port>,
}

impl NodeBuilder for Master {
    fn expr(
        &self,
        _nodes: &NodeArena,
        _function: &mut FunctionBuilder<'_>,
        _output: usize,
    ) -> Option<Handle<Expression>> {
        None
    }
}

impl Master {
    pub fn entry(&self, nodes: &NodeArena, module: &mut ModuleBuilder) -> Option<String> {
        let vs_main = self.vertex_entry(nodes, module)?;
        let fs_main = self.fragement_entry(nodes, module)?;

        module.push_entry_point(vs_main);
        module.push_entry_point(fs_main);

        Some(module.build())
    }

    pub fn vertex_entry(
        &self,
        nodes: &NodeArena,
        module: &mut ModuleBuilder,
    ) -> Option<EntryPoint> {
        let mut function = module.function();
        function.set_result(F32_4, Binding::BuiltIn(BuiltIn::Position));
        function.push_argument(
            String::from("vertex_index"),
            U32,
            Binding::BuiltIn(BuiltIn::VertexIndex),
        );

        let value = Some(nodes.expr(&mut function, self.position?)?);
        function.push_statement(Statement::Return { value });

        Some(EntryPoint {
            name: String::from("vs_main"),
            stage: ShaderStage::Vertex,
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: function.build(),
        })
    }

    pub fn fragement_entry(
        &self,
        nodes: &NodeArena,
        module: &mut ModuleBuilder,
    ) -> Option<EntryPoint> {
        let mut function = module.function();
        function.set_result(
            F32_4,
            Binding::Location {
                location: 0,
                interpolation: Some(Interpolation::Perspective),
                sampling: Some(Sampling::Center),
            },
        );

        let value = Some(nodes.expr(&mut function, self.color?)?);
        function.push_statement(Statement::Return { value });

        Some(EntryPoint {
            name: String::from("fs_main"),
            stage: ShaderStage::Fragment,
            early_depth_test: None,
            workgroup_size: [0; 3],
            function: function.build(),
        })
    }
}
