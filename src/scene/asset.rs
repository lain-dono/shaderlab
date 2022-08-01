use super::component::{ProxyTransform, ReflectProxy};
use super::context::ReflectEntityGetters;
use anyhow::Result;
use bevy::asset::{AssetEvent, AssetLoader, Assets, Handle, LoadContext, LoadedAsset};
use bevy::ecs::{
    entity::EntityMap,
    event::ManualEventReader,
    reflect::{ReflectComponent, ReflectMapEntities},
    system::Command,
};
use bevy::prelude::*;
use bevy::reflect::{
    serde::{ReflectDeserializer, ReflectSerializer},
    Reflect, TypeRegistry, TypeRegistryInternal, TypeUuid,
};
use bevy::utils::{tracing::error, BoxedFuture, HashMap};
use serde::{
    de::{DeserializeSeed, Error},
    ser::{SerializeSeq, SerializeStruct},
    Deserialize, Serialize,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Default)]
pub struct SceneMapping {
    pub entity: HashMap<u32, usize>,
    pub transform: HashMap<usize, GlobalTransform>,
}

impl SceneMapping {
    fn update(&mut self, scene: &ReflectScene) {
        self.entity.clear();
        self.transform.clear();

        self.entity.extend(
            scene
                .entities
                .iter()
                .enumerate()
                .map(|(index, entity)| (entity.entity, index)),
        );

        for parent_index in 0..scene.entities.len() {
            let parent = scene.entities.get(parent_index).unwrap();
            if parent.without::<Parent>() {
                if let Some(parent_transform) = parent.component_read::<ProxyTransform>() {
                    self.transform
                        .insert(parent_index, parent_transform.to_global());

                    self.recursive_propagate(scene, parent_index);
                }
            }
        }
    }

    fn recursive_propagate(&mut self, scene: &ReflectScene, parent_index: usize) -> Option<()> {
        let parent = scene.entities.get(parent_index).unwrap();
        let parent_transform = self.transform[&parent_index];

        for child in parent
            .children()?
            .iter()
            .filter_map(|entity| entity.downcast_ref::<Entity>())
        {
            if let Some(&index) = self.entity.get(&child.id()) {
                if let Some(child) = scene.entities.get(index) {
                    if let Some(transform) = child.component_read::<ProxyTransform>() {
                        self.transform
                            .insert(index, parent_transform.mul_transform(transform.to_local()));
                        self.recursive_propagate(scene, index);
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct ReflectSceneLoader {
    registry: TypeRegistry,
}

impl FromWorld for ReflectSceneLoader {
    fn from_world(world: &mut World) -> Self {
        let registry = world.resource::<TypeRegistry>();
        let registry = (*registry).clone();
        Self { registry }
    }
}

impl AssetLoader for ReflectSceneLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut ron_deserializer = ron::de::Deserializer::from_bytes(bytes)?;
            let scene_deserializer = SceneDeserializer {
                registry: &*self.registry.read(),
            };
            let scene = scene_deserializer.deserialize(&mut ron_deserializer)?;
            load_context.set_default_asset(LoadedAsset::new(scene));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["scene", "scene.ron"]
    }
}

#[derive(Debug, Default)]
pub(crate) struct InstanceInfo {
    entity_map: EntityMap,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InstanceId(Uuid);

impl InstanceId {
    fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Error, Debug)]
pub enum ReflectSceneSpawnError {
    #[error("scene contains the unregistered component `{type_name}`. consider adding `#[reflect(Component)]` to your type")]
    UnregisteredComponent { type_name: String },
    #[error("scene contains the unregistered type `{type_name}`. consider registering the type using `app.register_type::<T>()`")]
    UnregisteredType { type_name: String },
    #[error("scene does not exist")]
    NonExistentScene { handle: Handle<ReflectScene> },
}

#[derive(Default)]
pub struct ReflectSceneSpawner {
    pub(crate) scenes: HashMap<Handle<ReflectScene>, Vec<InstanceId>>,
    pub(crate) instances: HashMap<InstanceId, InstanceInfo>,
    pub(crate) event_reader: ManualEventReader<AssetEvent<ReflectScene>>,
    pub(crate) to_spawn: Vec<Handle<ReflectScene>>,
    pub(crate) to_despawn: Vec<Handle<ReflectScene>>,
    pub(crate) with_parent: Vec<(InstanceId, Entity)>,
}

impl ReflectSceneSpawner {
    pub fn spawn(&mut self, scene_handle: Handle<ReflectScene>) {
        self.to_spawn.push(scene_handle);
    }

    pub fn spawn_queued(&mut self, world: &mut World) -> Result<(), ReflectSceneSpawnError> {
        for scene_handle in std::mem::take(&mut self.to_spawn) {
            match self.spawn_sync(world, &scene_handle) {
                Ok(_) => (),
                Err(ReflectSceneSpawnError::NonExistentScene { .. }) => {
                    self.to_spawn.push(scene_handle)
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    pub fn spawn_sync(
        &mut self,
        world: &mut World,
        scene_handle: &Handle<ReflectScene>,
    ) -> Result<InstanceId, ReflectSceneSpawnError> {
        let mut info = InstanceInfo::default();
        Self::spawn_internal(world, scene_handle, &mut info.entity_map)?;

        let instance_id = InstanceId::new();
        self.instances.insert(instance_id, info);

        let spawned = self
            .scenes
            .entry(scene_handle.clone())
            .or_insert_with(Vec::new);

        spawned.push(instance_id);

        Ok(instance_id)
    }

    fn spawn_internal(
        world: &mut World,
        scene_handle: &Handle<ReflectScene>,
        entity_map: &mut EntityMap,
    ) -> Result<(), ReflectSceneSpawnError> {
        world.resource_scope(|world, scenes: Mut<Assets<ReflectScene>>| {
            let scene = scenes.get(scene_handle).ok_or_else(|| {
                ReflectSceneSpawnError::NonExistentScene {
                    handle: scene_handle.clone_weak(),
                }
            })?;
            scene.write_to_world(world, entity_map)
        })
    }

    pub fn update_spawned(
        &mut self,
        world: &mut World,
        scene_handles: &[Handle<ReflectScene>],
    ) -> Result<(), ReflectSceneSpawnError> {
        for scene_handle in scene_handles {
            if let Some(spawned_instances) = self.scenes.get(scene_handle) {
                for instance_id in spawned_instances.iter() {
                    if let Some(instance_info) = self.instances.get_mut(instance_id) {
                        Self::spawn_internal(world, scene_handle, &mut instance_info.entity_map)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn set_scene_instance_parent_sync(&mut self, world: &mut World) {
        let scenes_with_parent = std::mem::take(&mut self.with_parent);

        for (instance_id, parent) in scenes_with_parent {
            if let Some(instance) = self.instances.get(&instance_id) {
                for entity in instance.entity_map.values() {
                    // Add the `Parent` component to the scene root,
                    // and update the `Children` component of the scene parent
                    if !world
                        .get_entity(entity)
                        // This will filter only the scene root entity,
                        // as all other from the scene have a parent
                        .map(|entity| entity.contains::<Parent>())
                        // Default is true so that it won't run on an entity that wouldn't exist anymore
                        // this case shouldn't happen anyway
                        .unwrap_or(true)
                    {
                        AddChild {
                            parent,
                            child: entity,
                        }
                        .write(world);
                    }
                }
            } else {
                self.with_parent.push((instance_id, parent));
            }
        }
    }

    /// Check that an scene instance spawned previously is ready to use
    pub fn instance_is_ready(&self, instance_id: InstanceId) -> bool {
        self.instances.contains_key(&instance_id)
    }

    /// Get an iterator over the entities in an instance, once it's spawned
    pub fn iter_instance_entities(
        &'_ self,
        instance_id: InstanceId,
    ) -> Option<impl Iterator<Item = Entity> + '_> {
        self.instances
            .get(&instance_id)
            .map(|instance| instance.entity_map.values())
    }

    pub fn despawn(&mut self, scene_handle: Handle<ReflectScene>) {
        self.to_despawn.push(scene_handle);
    }

    pub fn despawn_queued(&mut self, world: &mut World) -> Result<(), ReflectSceneSpawnError> {
        let scenes_to_despawn = std::mem::take(&mut self.to_despawn);
        for scene_handle in scenes_to_despawn {
            self.despawn_sync(world, scene_handle)?;
        }
        Ok(())
    }

    pub fn despawn_sync(
        &mut self,
        world: &mut World,
        scene_handle: Handle<ReflectScene>,
    ) -> Result<(), ReflectSceneSpawnError> {
        if let Some(instance_ids) = self.scenes.get(&scene_handle) {
            for instance_id in instance_ids {
                if let Some(instance) = self.instances.get(instance_id) {
                    for entity in instance.entity_map.values() {
                        // Ignore the result, despawn only cares if it exists.
                        let _ = world.despawn(entity);
                    }
                }
            }
            self.scenes.remove(&scene_handle);
        }
        Ok(())
    }
}

/// A reflection-powered serializable representation of an entity and its components.
pub struct ReflectEntity {
    /// The transiently unique identifier of a corresponding `Entity`.
    pub entity: u32,
    /// A vector of boxed components that belong to the given entity and
    /// implement the `Reflect` trait.
    pub components: Vec<Box<dyn Reflect>>,
}

/// A collection of serializable dynamic entities, each with its own run-time defined set of components.
#[derive(Default, TypeUuid)]
#[uuid = "8ba344d5-02e9-4ee0-91fd-cd9d6f9b2233"]
pub struct ReflectScene {
    pub entities: Vec<ReflectEntity>,
}

impl ReflectScene {
    /// Create a new dynamic scene from a given world.
    pub fn from_world(world: &World, registry: &TypeRegistry) -> Self {
        let mut entities = Vec::new();
        let registry = registry.read();

        for archetype in world.archetypes().iter() {
            let offset = entities.len();

            // Create a new dynamic entity for each entity of the given archetype
            // and insert it into the dynamic scene.
            for entity in archetype.entities() {
                entities.push(ReflectEntity {
                    entity: entity.id(),
                    components: Vec::new(),
                });
            }

            // Add each reflection-powered component to the entity it belongs to.
            for component_id in archetype.components() {
                let reflect = world
                    .components()
                    .get_info(component_id)
                    .and_then(|info| registry.get(info.type_id().unwrap()))
                    .and_then(|registration| registration.data::<ReflectComponent>());

                if let Some(reflect) = reflect {
                    for (index, &entity) in archetype.entities().iter().enumerate() {
                        if let Some(component) = reflect.reflect(world, entity) {
                            entities[offset + index]
                                .components
                                .push(component.clone_value());
                        }
                    }
                }
            }
        }

        Self { entities }
    }

    /// Write the dynamic entities and their corresponding components to the given world.
    ///
    /// This method will return a `SceneSpawnError` if either a type is not registered
    /// or doesn't reflect the `Component` trait.
    pub fn write_to_world(
        &self,
        world: &mut World,
        entity_map: &mut EntityMap,
    ) -> Result<(), ReflectSceneSpawnError> {
        let registry = world.resource::<TypeRegistry>().clone();
        let registry = registry.read();

        for ReflectEntity { entity, components } in &self.entities {
            // Fetch the entity with the given entity id from the `entity_map`
            // or spawn a new entity with a transiently unique id if there is
            // no corresponding entry.
            let entity = *entity_map
                .entry(Entity::from_raw(*entity))
                .or_insert_with(|| world.spawn().id());

            // Apply/ add each component to the given entity.
            for component in components.iter().map(AsRef::as_ref) {
                let type_name = component.type_name();
                let registration = registry.get_with_name(type_name).ok_or_else(|| {
                    ReflectSceneSpawnError::UnregisteredType {
                        type_name: type_name.to_string(),
                    }
                })?;

                if let Some(proxy) = registration.data::<ReflectProxy>() {
                    proxy.resolve_and_add(world, entity, component);
                    continue;
                }

                let reflect = registration.data::<ReflectComponent>().ok_or_else(|| {
                    ReflectSceneSpawnError::UnregisteredComponent {
                        type_name: type_name.to_string(),
                    }
                })?;

                // If the entity already has the given component attached,
                // just apply the (possibly) new value, otherwise add the
                // component to the entity.
                let type_id = registration.type_id();

                if world.entity(entity).contains_type_id(type_id) {
                    reflect.apply(world, entity, component);
                } else {
                    reflect.insert(world, entity, component);
                }
            }
        }

        for registration in registry.iter() {
            if let Some(reflect) = registration.data::<ReflectMapEntities>() {
                reflect.map_entities(world, entity_map).unwrap();
            }
        }

        let mut context = world.resource_mut::<crate::util::anymap::AnyMap>();
        let mapping = context.get_or_insert(SceneMapping::default);
        mapping.update(self);

        Ok(())
    }
}

/// Serialize a given Rust data structure into rust object notation (ron).
pub fn serialize_ron<S>(serialize: S) -> Result<String, ron::Error>
where
    S: Serialize,
{
    let config = ron::ser::PrettyConfig::default()
        .decimal_floats(true)
        .struct_names(false)
        .indentor("  ".to_string())
        .new_line("\n".to_string());

    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::all())
        .to_string_pretty(&serialize, config)
}

pub struct SceneSerializer<'a> {
    pub scene: &'a ReflectScene,
    pub registry: &'a TypeRegistry,
}

impl<'a> SceneSerializer<'a> {
    pub fn new(scene: &'a ReflectScene, registry: &'a TypeRegistry) -> Self {
        Self { scene, registry }
    }
}

impl<'a> Serialize for SceneSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.scene.entities.len()))?;
        for entity in &self.scene.entities {
            state.serialize_element(&EntitySerializer {
                entity,
                registry: self.registry,
            })?;
        }
        state.end()
    }
}

pub struct EntitySerializer<'a> {
    pub entity: &'a ReflectEntity,
    pub registry: &'a TypeRegistry,
}

impl<'a> Serialize for EntitySerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct(ENTITY_STRUCT, 2)?;
        state.serialize_field(ENTITY_FIELD_ENTITY, &self.entity.entity)?;
        state.serialize_field(
            ENTITY_FIELD_COMPONENTS,
            &ComponentsSerializer {
                components: &self.entity.components,
                registry: self.registry,
            },
        )?;
        state.end()
    }
}

pub struct ComponentsSerializer<'a> {
    pub components: &'a [Box<dyn Reflect>],
    pub registry: &'a TypeRegistry,
}

impl<'a> Serialize for ComponentsSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.components.len()))?;
        for component in self.components.iter() {
            state.serialize_element(&ReflectSerializer::new(
                &**component,
                &*self.registry.read(),
            ))?;
        }
        state.end()
    }
}

pub struct SceneDeserializer<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneDeserializer<'a> {
    type Value = ReflectScene;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Self { registry } = self;
        let entities = deserializer.deserialize_seq(SceneEntitySeqVisitor { registry })?;
        Ok(Self::Value { entities })
    }
}

struct SceneEntitySeqVisitor<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> serde::de::Visitor<'de> for SceneEntitySeqVisitor<'a> {
    type Value = Vec<ReflectEntity>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("list of entities")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let Self { registry } = self;
        let mut entities = Vec::new();
        while let Some(entity) = seq.next_element_seed(SceneEntityDeserializer { registry })? {
            entities.push(entity);
        }
        Ok(entities)
    }
}

pub struct SceneEntityDeserializer<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> DeserializeSeed<'de> for SceneEntityDeserializer<'a> {
    type Value = ReflectEntity;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Self { registry } = self;
        deserializer.deserialize_struct(
            ENTITY_STRUCT,
            &[ENTITY_FIELD_ENTITY, ENTITY_FIELD_COMPONENTS],
            SceneEntityVisitor { registry },
        )
    }
}

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum EntityField {
    Entity,
    Components,
}

pub const ENTITY_STRUCT: &str = "Entity";
pub const ENTITY_FIELD_ENTITY: &str = "entity";
pub const ENTITY_FIELD_COMPONENTS: &str = "components";

struct SceneEntityVisitor<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> serde::de::Visitor<'de> for SceneEntityVisitor<'a> {
    type Value = ReflectEntity;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("entities")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let Self { registry } = self;
        let mut id = None;
        let mut components = None;
        while let Some(key) = map.next_key()? {
            match key {
                EntityField::Entity => {
                    if id.is_some() {
                        return Err(Error::duplicate_field(ENTITY_FIELD_ENTITY));
                    }
                    id = Some(map.next_value::<u32>()?);
                }
                EntityField::Components => {
                    if components.is_some() {
                        return Err(Error::duplicate_field(ENTITY_FIELD_COMPONENTS));
                    }

                    components = Some(map.next_value_seed(ComponentVecDeserializer { registry })?);
                }
            }
        }

        let entity = id
            .as_ref()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_ENTITY))?;

        let components = components
            .take()
            .ok_or_else(|| Error::missing_field(ENTITY_FIELD_COMPONENTS))?;
        Ok(ReflectEntity {
            entity: *entity,
            components,
        })
    }
}

pub struct ComponentVecDeserializer<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> DeserializeSeed<'de> for ComponentVecDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Self { registry } = self;
        deserializer.deserialize_seq(ComponentSeqVisitor { registry })
    }
}

struct ComponentSeqVisitor<'a> {
    pub registry: &'a TypeRegistryInternal,
}

impl<'a, 'de> serde::de::Visitor<'de> for ComponentSeqVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("list of components")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let Self { registry } = self;
        let mut dynamic_properties = Vec::new();
        while let Some(entity) = seq.next_element_seed(ReflectDeserializer::new(registry))? {
            dynamic_properties.push(entity);
        }
        Ok(dynamic_properties)
    }
}
