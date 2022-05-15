use bevy::utils::HashMap;
use std::any::{Any, TypeId};

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
