use std::any::TypeId;

use bevy::prelude::*;

use crate::*;

/// Type describing the target kind for a [`ReflectTarget`].
#[derive(Debug, Clone, Copy)]
pub enum ReflectKind {
    Component(Entity, TypeId),
    Resource(TypeId),
}

/// Type describing the path to a field on a target that can be operated on via reflection.
#[derive(Debug, Clone)]
pub struct ReflectTarget {
    pub kind: ReflectKind,
    pub field_path: String,
}

impl ReflectTarget {
    pub fn new_resource<T: Resource + Reflect>(field_path: impl Into<String>) -> Self {
        Self {
            kind: ReflectKind::Resource(TypeId::of::<T>()),
            field_path: field_path.into(),
        }
    }

    pub fn new_component<T: Component + Reflect>(
        entity: Entity,
        field_path: impl Into<String>,
    ) -> Self {
        Self {
            kind: ReflectKind::Component(entity, TypeId::of::<T>()),
            field_path: field_path.into(),
        }
    }
}

impl ReflectTarget {
    pub fn read_value<T: Reflect + Clone>(&self, world: &mut World) -> Result<T, ReflectError> {
        match self.kind {
            ReflectKind::Component(entity, type_id) => {
                reflect_component_read_path_from_world(world, entity, type_id, &self.field_path)
            }
            ReflectKind::Resource(type_id) => {
                reflect_resource_read_path(world, type_id, &self.field_path)
            }
        }
    }

    pub fn set_value<T: Reflect>(&self, world: &mut World, value: T) -> ReflectSetResult {
        match self.kind {
            ReflectKind::Component(entity, type_id) => {
                reflect_component_set_path(world, type_id, entity, &self.field_path, value)
            }
            ReflectKind::Resource(type_id) => {
                reflect_resource_set_path(world, type_id, &self.field_path, value)
            }
        }
    }

    pub fn toggle_reflect_enum(
        &self,
        world: &mut World,
        direction: EnumDirection,
    ) -> ReflectSetResult {
        match self.kind {
            ReflectKind::Component(entity, type_id) => reflect_component_toggle_enum_variant(
                world,
                type_id,
                entity,
                &self.field_path,
                direction,
                false,
            ),
            ReflectKind::Resource(type_id) => reflect_resource_toggle_enum_variant(
                world,
                type_id,
                &self.field_path,
                direction,
                false,
            ),
        }
    }

    pub fn read_enum_variant_name(&self, world: &mut World) -> Result<String, ReflectError> {
        match self.kind {
            ReflectKind::Component(entity, type_id) => {
                reflect_component_read_enum_variant_name_from_world(
                    world,
                    entity,
                    type_id,
                    &self.field_path,
                )
            }
            ReflectKind::Resource(type_id) => {
                reflect_resource_read_enum_variant_name(world, type_id, &self.field_path)
            }
        }
    }

    pub fn read_value_serialized(&self, world: &mut World) -> Result<String, ReflectError> {
        match self.kind {
            ReflectKind::Component(entity, type_id) => {
                reflect_component_read_path_serialized(world, entity, type_id, &self.field_path)
            }
            ReflectKind::Resource(type_id) => {
                reflect_resource_read_path_serialized(world, type_id, &self.field_path)
            }
        }
    }

    pub fn set_value_serialized(&self, world: &mut World, value: &str) -> ReflectSetResult {
        match self.kind {
            ReflectKind::Component(entity, type_id) => reflect_component_set_path_serialized(
                world,
                entity,
                type_id,
                &self.field_path,
                value,
            ),
            ReflectKind::Resource(type_id) => {
                reflect_resource_set_path_serialized(world, type_id, &self.field_path, value)
            }
        }
    }

    pub fn partial_eq_serialized(
        &self,
        world: &mut World,
        serialized_value: &str,
    ) -> Result<bool, ReflectError> {
        match self.kind {
            ReflectKind::Component(entity, type_id) => reflect_component_partial_eq_serialized(
                world,
                entity,
                type_id,
                &self.field_path,
                serialized_value,
            ),
            ReflectKind::Resource(type_id) => reflect_resource_partial_eq_serialized(
                world,
                type_id,
                &self.field_path,
                serialized_value,
            ),
        }
    }
}
