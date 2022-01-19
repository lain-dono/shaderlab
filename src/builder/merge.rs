use naga::{
    Arena, AtomicFunction, Block, Constant, ConstantInner, EntryPoint, Expression, Function,
    FunctionArgument, FunctionResult, GlobalVariable, Handle, ImageQuery, LocalVariable, Module,
    Span, Statement, StructMember, SwitchCase, Type, TypeInner, UniqueArena,
};
use std::collections::HashMap;

type Types = HashMap<Handle<Type>, Handle<Type>>;
type Constants = HashMap<Handle<Constant>, Handle<Constant>>;
type GlobalVariables = HashMap<Handle<GlobalVariable>, Handle<GlobalVariable>>;
type LocalVariables = HashMap<Handle<LocalVariable>, Handle<LocalVariable>>;
type Functions = HashMap<Handle<Function>, Handle<Function>>;
type Expressions = HashMap<Handle<Expression>, Handle<Expression>>;

pub fn merge_module(dst: &mut Module, src: &Module) {
    let types = merge_types(&mut dst.types, &src.types);
    let constants = merge_constants(&mut dst.constants, &src.constants, &types);
    let global_variables = merge_global_variables(
        &mut dst.global_variables,
        &src.global_variables,
        &types,
        &constants,
    );
    let functions = merge_functions(
        &mut dst.functions,
        &src.functions,
        &types,
        &constants,
        &global_variables,
    );
    merge_entry_points(
        &mut dst.entry_points,
        &src.entry_points,
        &types,
        &constants,
        &global_variables,
        &functions,
    );
}

fn merge_types(
    dst: &mut UniqueArena<Type>,
    src: &UniqueArena<Type>,
) -> HashMap<Handle<Type>, Handle<Type>> {
    let mut types = HashMap::default();
    for (src, ty) in src.iter() {
        let name = ty.name.clone();
        let inner = match ty.inner {
            TypeInner::Scalar { kind, width } => TypeInner::Scalar { kind, width },
            TypeInner::Vector { size, kind, width } => TypeInner::Vector { size, kind, width },
            TypeInner::Matrix {
                columns,
                rows,
                width,
            } => TypeInner::Matrix {
                columns,
                rows,
                width,
            },
            TypeInner::Atomic { kind, width } => TypeInner::Atomic { kind, width },
            TypeInner::Pointer { base, class } => TypeInner::Pointer {
                base: types[&base],
                class,
            },
            TypeInner::ValuePointer {
                size,
                kind,
                width,
                class,
            } => TypeInner::ValuePointer {
                size,
                kind,
                width,
                class,
            },
            TypeInner::Array { base, size, stride } => TypeInner::Array {
                base: types[&base],
                size,
                stride,
            },
            TypeInner::Struct { ref members, span } => TypeInner::Struct {
                span,
                members: members
                    .iter()
                    .map(|member| StructMember {
                        name: member.name.clone(),
                        ty: types[&member.ty],
                        binding: member.binding.clone(),
                        offset: member.offset,
                    })
                    .collect(),
            },
            TypeInner::Image {
                dim,
                arrayed,
                class,
            } => TypeInner::Image {
                dim,
                arrayed,
                class,
            },
            TypeInner::Sampler { comparison } => TypeInner::Sampler { comparison },
        };
        types.insert(src, dst.insert(Type { name, inner }, Span::default()));
    }
    types
}

fn merge_constants(dst: &mut Arena<Constant>, src: &Arena<Constant>, types: &Types) -> Constants {
    let mut constants: Constants = HashMap::default();
    for (src, constant) in src.iter() {
        let constant = Constant {
            name: constant.name.clone(),
            specialization: constant.specialization,
            inner: match constant.inner {
                ConstantInner::Scalar { width, value } => ConstantInner::Scalar { width, value },
                ConstantInner::Composite {
                    ref ty,
                    ref components,
                } => ConstantInner::Composite {
                    ty: types[ty],
                    components: components.iter().map(|c| constants[c]).collect(),
                },
            },
        };
        constants.insert(src, dst.append(constant, Span::default()));
    }
    constants
}

fn merge_global_variables(
    dst: &mut Arena<GlobalVariable>,
    src: &Arena<GlobalVariable>,
    types: &Types,
    constants: &Constants,
) -> GlobalVariables {
    src.iter()
        .map(|(src, var)| {
            let variable = GlobalVariable {
                name: var.name.clone(),
                class: var.class,
                binding: var.binding.clone(),
                ty: types[&var.ty],
                init: var.init.map(|init| constants[&init]),
            };
            (src, dst.append(variable, Span::default()))
        })
        .collect()
}

fn merge_functions(
    dst: &mut Arena<Function>,
    src: &Arena<Function>,
    types: &Types,
    constants: &Constants,
    global_variables: &GlobalVariables,
) -> Functions {
    let mut functions: Functions = HashMap::default();
    for (src, function) in src.iter() {
        let function = clone_function(function, types, constants, global_variables, &functions);
        functions.insert(src, dst.append(function, Span::default()));
    }
    functions
}

fn merge_entry_points(
    dst: &mut Vec<EntryPoint>,
    src: &[EntryPoint],
    types: &Types,
    constants: &Constants,
    global_variables: &GlobalVariables,
    functions: &Functions,
) {
    dst.extend(src.iter().map(|entry| EntryPoint {
        name: entry.name.clone(),
        stage: entry.stage,
        early_depth_test: entry.early_depth_test,
        workgroup_size: entry.workgroup_size,
        function: clone_function(
            &entry.function,
            types,
            constants,
            global_variables,
            functions,
        ),
    }))
}

fn clone_function(
    src: &Function,
    types: &Types,
    constants: &Constants,
    global_variables: &GlobalVariables,
    functions: &Functions,
) -> Function {
    let mut local_variables: LocalVariables = HashMap::default();
    let mut expressions: Expressions = HashMap::default();

    Function {
        name: src.name.clone(),
        arguments: src
            .arguments
            .iter()
            .map(|arg| FunctionArgument {
                name: arg.name.clone(),
                ty: types[&arg.ty],
                binding: arg.binding.clone(),
            })
            .collect(),
        result: src.result.as_ref().map(|result| FunctionResult {
            ty: types[&result.ty],
            binding: result.binding.clone(),
        }),
        local_variables: {
            let mut loc = Arena::new();
            for (src, var) in src.local_variables.iter() {
                let dst = loc.append(
                    LocalVariable {
                        name: var.name.clone(),
                        ty: types[&var.ty],
                        init: var.init.map(|init| constants[&init]),
                    },
                    Span::default(),
                );
                local_variables.insert(src, dst);
            }
            loc
        },
        expressions: {
            let mut exprs = Arena::new();
            for (src, expr) in src.expressions.iter() {
                let expr = match expr {
                    Expression::Access { base, index } => Expression::Access {
                        base: expressions[base],
                        index: expressions[index],
                    },
                    Expression::AccessIndex { base, index } => Expression::AccessIndex {
                        base: expressions[base],
                        index: *index,
                    },
                    Expression::Constant(handle) => Expression::Constant(constants[handle]),
                    Expression::Splat { size, value } => Expression::Splat {
                        size: *size,
                        value: expressions[value],
                    },
                    Expression::Swizzle {
                        size,
                        vector,
                        pattern,
                    } => Expression::Swizzle {
                        size: *size,
                        vector: expressions[vector],
                        pattern: *pattern,
                    },
                    Expression::Compose { ty, components } => Expression::Compose {
                        ty: types[ty],
                        components: components.iter().map(|expr| expressions[expr]).collect(),
                    },
                    Expression::FunctionArgument(i) => Expression::FunctionArgument(*i),
                    Expression::GlobalVariable(global) => {
                        Expression::GlobalVariable(global_variables[global])
                    }
                    Expression::LocalVariable(v) => Expression::LocalVariable(local_variables[v]),
                    Expression::Load { pointer } => Expression::Load {
                        pointer: expressions[pointer],
                    },
                    Expression::ImageSample {
                        image,
                        sampler,
                        coordinate,
                        array_index,
                        offset,
                        level,
                        depth_ref,
                        gather,
                    } => Expression::ImageSample {
                        image: expressions[image],
                        sampler: expressions[sampler],
                        coordinate: expressions[coordinate],
                        array_index: array_index.map(|expr| expressions[&expr]),
                        offset: offset.map(|c| constants[&c]),
                        level: *level,
                        depth_ref: depth_ref.map(|expr| expressions[&expr]),
                        gather: *gather,
                    },
                    Expression::ImageLoad {
                        image,
                        coordinate,
                        array_index,
                        index,
                    } => Expression::ImageLoad {
                        image: expressions[image],
                        coordinate: expressions[coordinate],
                        array_index: array_index.map(|expr| expressions[&expr]),
                        index: index.map(|expr| expressions[&expr]),
                    },
                    Expression::ImageQuery { image, query } => Expression::ImageQuery {
                        image: expressions[image],
                        query: match query {
                            ImageQuery::Size { level } => ImageQuery::Size {
                                level: level.map(|expr| expressions[&expr]),
                            },
                            ImageQuery::NumLevels => ImageQuery::NumLevels,
                            ImageQuery::NumLayers => ImageQuery::NumLayers,
                            ImageQuery::NumSamples => ImageQuery::NumSamples,
                        },
                    },
                    Expression::Unary { op, expr } => Expression::Unary {
                        op: *op,
                        expr: expressions[expr],
                    },
                    Expression::Binary { op, left, right } => Expression::Binary {
                        op: *op,
                        left: expressions[left],
                        right: expressions[right],
                    },
                    Expression::Select {
                        condition,
                        accept,
                        reject,
                    } => Expression::Select {
                        condition: expressions[condition],
                        accept: expressions[accept],
                        reject: expressions[reject],
                    },
                    Expression::Derivative { axis, expr } => Expression::Derivative {
                        axis: *axis,
                        expr: expressions[expr],
                    },
                    Expression::Relational { fun, argument } => Expression::Relational {
                        fun: *fun,
                        argument: expressions[argument],
                    },
                    Expression::Math {
                        fun,
                        arg,
                        arg1,
                        arg2,
                        arg3,
                    } => Expression::Math {
                        fun: *fun,
                        arg: expressions[arg],
                        arg1: arg1.map(|expr| expressions[&expr]),
                        arg2: arg2.map(|expr| expressions[&expr]),
                        arg3: arg3.map(|expr| expressions[&expr]),
                    },
                    Expression::As {
                        expr,
                        kind,
                        convert,
                    } => Expression::As {
                        expr: expressions[expr],
                        kind: *kind,
                        convert: *convert,
                    },
                    Expression::CallResult(fun) => Expression::CallResult(functions[fun]),
                    Expression::AtomicResult {
                        kind,
                        width,
                        comparison,
                    } => Expression::AtomicResult {
                        kind: *kind,
                        width: *width,
                        comparison: *comparison,
                    },
                    Expression::ArrayLength(expr) => Expression::ArrayLength(expressions[expr]),
                };

                expressions.insert(src, exprs.append(expr, Span::default()));
            }
            exprs
        },
        named_expressions: src
            .named_expressions
            .iter()
            .map(|(expr, name)| (expressions[expr], name.clone()))
            .collect(),
        body: clone_block(&src.body, &expressions, functions),
    }
}

fn clone_block(block: &Block, expressions: &Expressions, functions: &Functions) -> Block {
    let block = block.iter().map(|stmt| match stmt {
        Statement::Emit(range) => Statement::Emit(range.clone()),
        Statement::Block(block) => Statement::Block(clone_block(block, expressions, functions)),
        Statement::If {
            condition,
            accept,
            reject,
        } => Statement::If {
            condition: expressions[condition],
            accept: clone_block(accept, expressions, functions),
            reject: clone_block(reject, expressions, functions),
        },
        Statement::Switch { selector, cases } => Statement::Switch {
            selector: expressions[selector],
            cases: cases
                .iter()
                .map(|sc| SwitchCase {
                    value: sc.value.clone(), // todo
                    body: clone_block(&sc.body, expressions, functions),
                    fall_through: sc.fall_through,
                })
                .collect(),
        },
        Statement::Loop { body, continuing } => Statement::Loop {
            body: clone_block(body, expressions, functions),
            continuing: clone_block(continuing, expressions, functions),
        },
        Statement::Break => Statement::Break,
        Statement::Continue => Statement::Continue,
        Statement::Return { value } => Statement::Return {
            value: value.map(|expr| expressions[&expr]),
        },
        Statement::Kill => Statement::Kill,
        Statement::Barrier(barrier) => Statement::Barrier(*barrier),
        Statement::Store { pointer, value } => Statement::Store {
            pointer: expressions[pointer],
            value: expressions[value],
        },
        Statement::ImageStore {
            image,
            coordinate,
            array_index,
            value,
        } => Statement::ImageStore {
            image: expressions[image],
            coordinate: expressions[coordinate],
            array_index: array_index.map(|expr| expressions[&expr]),
            value: expressions[value],
        },
        Statement::Atomic {
            pointer,
            fun,
            value,
            result,
        } => Statement::Atomic {
            pointer: expressions[pointer],
            fun: match fun {
                AtomicFunction::Add => AtomicFunction::Add,
                AtomicFunction::Subtract => AtomicFunction::Subtract,
                AtomicFunction::And => AtomicFunction::And,
                AtomicFunction::ExclusiveOr => AtomicFunction::ExclusiveOr,
                AtomicFunction::InclusiveOr => AtomicFunction::InclusiveOr,
                AtomicFunction::Min => AtomicFunction::Min,
                AtomicFunction::Max => AtomicFunction::Max,
                AtomicFunction::Exchange { compare } => AtomicFunction::Exchange {
                    compare: compare.map(|expr| expressions[&expr]),
                },
            },
            value: expressions[value],
            result: expressions[result],
        },
        Statement::Call {
            function,
            arguments,
            result,
        } => Statement::Call {
            function: functions[function],
            arguments: arguments.iter().map(|expr| expressions[expr]).collect(),
            result: result.map(|expr| expressions[&expr]),
        },
    });

    Block::from_vec(block.collect())
}
