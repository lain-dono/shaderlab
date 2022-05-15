use crate::asset::{ReflectEntity, ReflectScene};
use crate::util::anymap::AnyMap;
use bevy::prelude::{AssetServer, Children};
use bevy::reflect::{
    FromReflect, GetField, GetTupleField, GetTupleStructField, Reflect, ReflectMut, ReflectRef,
    TypeRegistry,
};

pub struct SelectedEntityId(usize);

pub struct EditorContext<'a> {
    pub scene: &'a mut ReflectScene,
    pub state: &'a mut AnyMap,
    pub types: &'a TypeRegistry,
    pub assets: &'a mut AssetServer,
}

impl<'a> EditorContext<'a> {
    pub fn select(&mut self, entity: usize) {
        self.state.insert(SelectedEntityId(entity));
    }

    pub fn selected_index(&mut self, lock: Option<usize>) -> Option<usize> {
        lock.or_else(|| unsafe {
            self.state
                .query::<Option<&SelectedEntityId>>()
                .map(|id| id.0)
        })
    }

    pub fn find_selected(&mut self, lock: Option<usize>) -> Option<EntityEditor> {
        let index = self.selected_index(lock)?;
        self.get(index)
    }

    pub fn get(&mut self, index: usize) -> Option<EntityEditor> {
        self.scene
            .entities
            .get_mut(index)
            .map(|entity| EntityEditor {
                index,
                entity,
                types: self.types,
                assets: self.assets,
            })
    }
}

pub struct EntityEditor<'a> {
    pub index: usize,
    pub types: &'a TypeRegistry,
    pub entity: &'a mut ReflectEntity,
    pub assets: &'a mut AssetServer,
}

impl<'a> EntityEditor<'a> {
    #[inline]
    pub fn has<T: Reflect>(&self) -> bool {
        let type_name = std::any::type_name::<T>();
        self.entity
            .components
            .iter()
            .any(|c| c.type_name() == type_name)
    }

    #[inline]
    pub fn without<T: Reflect>(&self) -> bool {
        let type_name = std::any::type_name::<T>();
        self.entity
            .components
            .iter()
            .all(|c| c.type_name() != type_name)
    }

    #[inline]
    pub fn children(&self) -> Option<&(dyn bevy::reflect::List + 'static)> {
        let type_name = std::any::type_name::<Children>();
        let children = self
            .entity
            .components
            .iter()
            .find(|c| c.type_name() == type_name)?;

        let reflect = children.reflect_ref();

        let reflect = if let ReflectRef::TupleStruct(reflect) = reflect {
            reflect
        } else {
            return None;
        };

        if let ReflectRef::List(list) = reflect.field(0)?.reflect_ref() {
            Some(list)
        } else {
            None
        }
    }
}

impl ReflectEntityGetters for ReflectEntity {
    fn component_ref<R: Reflect>(&self) -> Option<ReflectRef> {
        let type_name = std::any::type_name::<R>();
        self.components
            .iter()
            .find(|c| c.type_name() == type_name)
            .map(|r| r.reflect_ref())
    }

    fn component_mut<R: Reflect>(&mut self) -> Option<ReflectMut> {
        let type_name = std::any::type_name::<R>();
        self.components
            .iter_mut()
            .find(|c| c.type_name() == type_name)
            .map(|r| r.reflect_mut())
    }

    fn component_read<R: FromReflect + Sized>(&self) -> Option<R> {
        let type_name = std::any::type_name::<R>();
        self.components
            .iter()
            .find(|c| c.type_name() == type_name)
            .and_then(|r| R::from_reflect(r.as_ref()))
    }
}

pub trait ReflectEntityGetters {
    fn component_ref<R: Reflect>(&self) -> Option<ReflectRef>;
    fn component_mut<R: Reflect>(&mut self) -> Option<ReflectMut>;
    fn component_read<R: FromReflect>(&self) -> Option<R>;

    fn has<T: Reflect>(&self) -> bool {
        self.component_ref::<T>().is_some()
    }

    fn without<T: Reflect>(&self) -> bool {
        self.component_ref::<T>().is_none()
    }

    fn children(&self) -> Option<&(dyn bevy::reflect::List + 'static)> {
        let reflect = if let ReflectRef::TupleStruct(reflect) = self.component_ref::<Children>()? {
            reflect
        } else {
            return None;
        };
        if let ReflectRef::List(list) = reflect.field(0)?.reflect_ref() {
            Some(list)
        } else {
            None
        }
    }

    fn tuple_field_ref<R: Reflect, F: Reflect>(&self, index: usize) -> Option<&F> {
        match self.component_ref::<R>()? {
            ReflectRef::Tuple(s) => s.get_field(index),
            ReflectRef::TupleStruct(s) => s.get_field(index),
            _ => None,
        }
    }

    fn tuple_field_mut<R: Reflect, F: Reflect>(&mut self, index: usize) -> Option<&mut F> {
        match self.component_mut::<R>()? {
            ReflectMut::Tuple(s) => s.get_field_mut(index),
            ReflectMut::TupleStruct(s) => s.get_field_mut(index),
            _ => None,
        }
    }

    fn struct_field_ref<R: Reflect, F: Reflect>(&self, name: &str) -> Option<&F> {
        match self.component_ref::<R>()? {
            ReflectRef::Struct(s) => s.get_field(name),
            _ => None,
        }
    }

    fn struct_field_mut<R: Reflect, F: Reflect>(&mut self, name: &str) -> Option<&mut F> {
        match self.component_mut::<R>()? {
            ReflectMut::Struct(s) => s.field_mut(name).and_then(|c| c.downcast_mut()),
            _ => None,
        }
    }
}
