use crate::asset::{ReflectEntity, ReflectScene};
use bevy::prelude::{AssetServer, Children};
use bevy::reflect::{
    GetField, GetTupleField, GetTupleStructField, Reflect, ReflectMut, ReflectRef, TypeRegistry,
};
use bevy::utils::HashMap;
use std::any::{Any, TypeId};

pub struct SelectedEntityId(usize);

pub struct EditorContext<'a> {
    pub scene: &'a mut ReflectScene,
    pub state: &'a mut AnyMap,
    pub type_registry: &'a TypeRegistry,
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
                type_registry: self.type_registry,
                assets: self.assets,
            })
    }
}

pub struct EntityEditor<'a> {
    pub index: usize,
    pub type_registry: &'a TypeRegistry,
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

    pub fn component_ref<R: Reflect>(&self) -> Option<ReflectRef> {
        let type_name = std::any::type_name::<R>();
        self.entity
            .components
            .iter()
            .find(|c| c.type_name() == type_name)
            .map(|r| r.reflect_ref())
    }

    pub fn component_mut<R: Reflect>(&mut self) -> Option<ReflectMut> {
        let type_name = std::any::type_name::<R>();
        self.entity
            .components
            .iter_mut()
            .find(|c| c.type_name() == type_name)
            .map(|r| r.reflect_mut())
    }

    pub fn tuple_field_ref<R: Reflect, F: Reflect>(&self, index: usize) -> Option<&F> {
        match self.component_ref::<R>()? {
            ReflectRef::Tuple(s) => s.get_field(index),
            ReflectRef::TupleStruct(s) => s.get_field(index),
            _ => None,
        }
    }

    pub fn tuple_field_mut<R: Reflect, F: Reflect>(&mut self, index: usize) -> Option<&mut F> {
        match self.component_mut::<R>()? {
            ReflectMut::Tuple(s) => s.get_field_mut(index),
            ReflectMut::TupleStruct(s) => s.get_field_mut(index),
            _ => None,
        }
    }

    pub fn struct_field_ref<R: Reflect, F: Reflect>(&self, name: &str) -> Option<&F> {
        match self.component_ref::<R>()? {
            ReflectRef::Struct(s) => s.get_field(name),
            _ => None,
        }
    }

    pub fn struct_field_mut<R: Reflect, F: Reflect>(&mut self, name: &str) -> Option<&mut F> {
        match self.component_mut::<R>()? {
            ReflectMut::Struct(s) => s.field_mut(name).and_then(|c| c.downcast_mut()),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct AnyMap {
    data: HashMap<TypeId, Box<dyn Any>>,
}

unsafe impl Send for AnyMap {}
unsafe impl Sync for AnyMap {}

impl AnyMap {
    #[inline]
    pub fn has<R: Any>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<R>())
    }

    #[inline]
    #[track_caller]
    pub fn insert<R: Any>(&mut self, res: R) -> Option<Box<R>> {
        self.data
            .insert(TypeId::of::<R>(), Box::new(res))
            .map(|r| unsafe { r.downcast::<R>().unwrap_unchecked() })
    }

    #[inline]
    pub fn remove<R: Any>(&mut self) -> Option<Box<R>> {
        self.data
            .remove(&TypeId::of::<R>())
            .map(|r| unsafe { r.downcast::<R>().unwrap_unchecked() })
    }

    /// # Safety
    ///
    /// Can produce multiple mutable references.
    #[inline]
    #[track_caller]
    pub unsafe fn query<'a, Q: AnyMapQuery<'a>>(&'a mut self) -> Q::Result {
        Q::query(self)
    }
}

pub trait AnyMapQuery<'a> {
    type Result;

    fn query(map: &'a mut AnyMap) -> Self::Result;
}

impl<'a, T: Any> AnyMapQuery<'a> for Option<&'a T> {
    type Result = Option<&'a T>;

    #[track_caller]
    fn query(map: &'a mut AnyMap) -> Self::Result {
        map.data
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }
}

impl<'a, T: Any> AnyMapQuery<'a> for &'a T {
    type Result = &'a T;

    #[track_caller]
    fn query(map: &'a mut AnyMap) -> Self::Result {
        <Option<&'a T> as AnyMapQuery<'a>>::query(map).unwrap()
    }
}

impl<'a, T: Any> AnyMapQuery<'a> for Option<&'a mut T> {
    type Result = Option<&'a mut T>;

    #[track_caller]
    fn query(map: &'a mut AnyMap) -> Self::Result {
        map.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut::<T>())
    }
}

impl<'a, T: Any> AnyMapQuery<'a> for &'a mut T {
    type Result = &'a mut T;

    #[track_caller]
    fn query(map: &'a mut AnyMap) -> Self::Result {
        <Option<&'a mut T> as AnyMapQuery<'a>>::query(map).unwrap()
    }
}

macro_rules! impl_tuple {
    ($($type:ident),+) => {
        impl<'a, $($type: AnyMapQuery<'a>),+> AnyMapQuery<'a> for ($($type,)+) {
            type Result = ($($type::Result,)+);

            #[track_caller]
            fn query(map: &'a mut AnyMap) -> Self::Result {
                unsafe {
                    (
                        $( $type::query(&mut *(map as *mut _)), )+
                    )
                }
            }
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
impl_tuple!(A, B, C, D, E, F, G, H, I);
impl_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

#[test]
fn query() {
    struct Foo;
    struct Bar;

    let mut store = AnyMap::default();

    store.insert(Foo);
    store.insert(Bar);

    let (_foo, _bar, _opt_foo) = unsafe { store.query::<(&Foo, &mut Bar, Option<&Foo>)>() };
}
