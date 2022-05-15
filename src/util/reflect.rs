use bevy::prelude::*;
use bevy::reflect::*;

pub fn debug_reflect(level: usize, reflect: &dyn Reflect) {
    if let Some(value) = reflect.downcast_ref::<f32>() {
        return println!("<{}> {}", reflect.type_name(), value);
    }

    if let Some(value) = reflect.downcast_ref::<Vec3>() {
        return println!("<{}> {}", reflect.type_name(), value);
    }
    if let Some(value) = reflect.downcast_ref::<Quat>() {
        return println!("<{}> {}", reflect.type_name(), value);
    }
    if let Some(value) = reflect.downcast_ref::<String>() {
        return println!("<{}> {}", reflect.type_name(), value);
    }

    println!();
    for _ in 0..level {
        print!("  ");
    }

    match reflect.reflect_ref() {
        ReflectRef::Struct(reflect) => debug_struct(level + 1, reflect),
        ReflectRef::TupleStruct(reflect) => debug_tuple_struct(level + 1, reflect),
        ReflectRef::Tuple(reflect) => debug_tuple(level + 1, reflect),
        ReflectRef::List(reflect) => debug_list(level + 1, reflect),
        ReflectRef::Map(reflect) => debug_map(level + 1, reflect),
        ReflectRef::Value(reflect) => println!("{}", reflect.type_name()),
    }
}

pub fn debug_struct(level: usize, reflect: &(dyn Struct + 'static)) {
    println!("{} {{", reflect.type_name());
    for (index, field) in reflect.iter_fields().enumerate() {
        if let Some(name) = reflect.name_at(index) {
            for _ in 0..level {
                print!("  ");
            }
            print!("{}: ", name);
        }
        debug_reflect(level + 1, field);
    }
    for _ in 0..level.saturating_sub(1) {
        print!("  ");
    }
    println!("}}");
}

pub fn debug_tuple_struct(level: usize, reflect: &(dyn TupleStruct + 'static)) {
    println!("tuple_struct<{}>", reflect.type_name());
    for index in 0..reflect.field_len() {
        let field = reflect.field(index).unwrap();
        debug_reflect(level + 1, field);
    }
}

pub fn debug_tuple(level: usize, reflect: &(dyn Tuple + 'static)) {
    println!("tuple<{}>", reflect.type_name());
    for index in 0..reflect.field_len() {
        let field = reflect.field(index).unwrap();
        debug_reflect(level + 1, field);
    }
}

pub fn debug_list(_level: usize, reflect: &(dyn List + 'static)) {
    println!("list<{}>", reflect.type_name());
}

pub fn debug_map(_level: usize, reflect: &(dyn Map + 'static)) {
    println!("map<{}>", reflect.type_name());
}
