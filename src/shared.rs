use std::any::TypeId;

use bevy::{
    prelude::*,
    reflect::{
        serde::{ReflectDeserializer, ReflectSerializer},
        DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant, TypeRegistry, VariantInfo,
    },
    scene::ron,
};
use serde::de::DeserializeSeed;

use crate::ReflectError;

pub fn deserialize_reflect_value(
    world: &mut World,
    serialized_value: &str,
) -> Result<Box<dyn Reflect>, ReflectError> {
    let app_type_registry = world.resource_mut::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    // De-serialize the value
    let reflect_deserializer = ReflectDeserializer::new(&type_registry);
    let mut deserializer = ron::de::Deserializer::from_str(serialized_value)
        .map_err(|err| ReflectError::Deserialize(format!("{err:?}")))?;
    reflect_deserializer
        .deserialize(&mut deserializer)
        .map_err(|err| ReflectError::Deserialize(format!("{err:?}")))
}

pub fn serialize_reflect_value(
    type_registry: &TypeRegistry,
    value: &dyn Reflect,
) -> Result<String, ReflectError> {
    // By default, all derived `Reflect` types can be Serialized using serde. No need to derive
    // Serialize!
    let serializer = ReflectSerializer::new(value, type_registry);
    ron::ser::to_string(&serializer).map_err(|err| ReflectError::Serialize(format!("{err:?}")))
}

pub fn serialize_reflect_value_from_world<T: Reflect>(
    world: &mut World,
    value: &T,
) -> Result<String, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();
    serialize_reflect_value(&type_registry, value)
}

/// Returns the default value for a reflectable type id, if it can.
/// CREDIT: Copied from `bevy-inspector-egui`
fn get_default_value_for(
    type_registry: &TypeRegistry,
    type_id: TypeId,
) -> Option<Box<dyn Reflect>> {
    type_registry
        .get_type_data::<ReflectDefault>(type_id)
        .map(|reflect_default| reflect_default.default())
}

/// Utility that constructs `DynamicEnum` with the default value for the variant.
/// CREDIT: Copied from `bevy-inspector-egui`
pub fn construct_default_enum_variant(
    variant: &VariantInfo,
    type_registry: &TypeRegistry,
) -> Result<DynamicEnum, ReflectError> {
    let dynamic_variant = match variant {
        VariantInfo::Struct(struct_info) => {
            let mut dynamic_struct = DynamicStruct::default();
            for field in struct_info.iter() {
                let field_default_value = get_default_value_for(type_registry, field.type_id())
                    .ok_or(ReflectError::NoDefaultValue)?;
                dynamic_struct.insert_boxed(field.name(), field_default_value);
            }
            DynamicVariant::Struct(dynamic_struct)
        }
        VariantInfo::Tuple(tuple_info) => {
            let mut dynamic_tuple = DynamicTuple::default();
            for field in tuple_info.iter() {
                let field_default_value = get_default_value_for(type_registry, field.type_id())
                    .ok_or(ReflectError::NoDefaultValue)?;
                dynamic_tuple.insert_boxed(field_default_value);
            }
            DynamicVariant::Tuple(dynamic_tuple)
        }
        VariantInfo::Unit(_) => DynamicVariant::Unit,
    };

    let dynamic_enum = DynamicEnum::new(variant.name(), dynamic_variant);
    Ok(dynamic_enum)
}

/// Utility that tries to read the `TypeId` of a type path from a `TypeRegistry`.
///
/// Returns None if the type is not registered.
pub fn get_type_id_for_type_path(type_registry: &TypeRegistry, type_path: &str) -> Option<TypeId> {
    type_registry
        .get_with_type_path(type_path)
        .map(|type_registration| type_registration.type_id())
}

/// Utility that tries to read the `TypeId` of a type path from the world.
///
/// Panics if the world does not contain a `AppTypeRegistry` component.
///
/// Returns None if the type is not registered.
pub fn get_type_id_for_type_path_from_world(world: &World, type_path: &str) -> Option<TypeId> {
    let type_registry = world.resource::<AppTypeRegistry>().read();
    get_type_id_for_type_path(&type_registry, type_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Reflect, Default, Clone)]
    #[reflect(Resource)]
    struct ResourceA;

    #[derive(Component, Reflect, Default, Clone)]
    #[reflect(Component)]
    struct ComponentA;

    #[derive(Component, Reflect, Default, Clone)]
    #[reflect(Component)]
    struct NonRegisteredComponent;

    /// Test utility that creates a new world and registers the test types
    fn create_world() -> World {
        let mut world = World::new();
        world.init_resource::<AppTypeRegistry>();

        let type_registry = world.resource_mut::<AppTypeRegistry>();
        type_registry.write().register::<ResourceA>();
        type_registry.write().register::<ComponentA>();

        world
    }

    #[test]
    fn get_type_id_for_type_path_from_world_works() {
        let world = create_world();

        // Test we can read the `TypeId` of `ResourceA`
        assert_eq!(
            get_type_id_for_type_path_from_world(
                &world,
                "bevy_reflect_utils::shared::tests::ResourceA"
            ),
            Some(TypeId::of::<ResourceA>())
        );

        // Test we can read the `TypeId` of `ComponentA`
        assert_eq!(
            get_type_id_for_type_path_from_world(
                &world,
                "bevy_reflect_utils::shared::tests::ComponentA"
            ),
            Some(TypeId::of::<ComponentA>())
        );
    }

    #[test]
    fn get_type_id_for_type_path_works() {
        let world = create_world();
        let type_registry = world.resource::<AppTypeRegistry>().read();

        // Test we can read the `TypeId` of `ResourceA`
        assert_eq!(
            get_type_id_for_type_path(
                &type_registry,
                "bevy_reflect_utils::shared::tests::ResourceA"
            ),
            Some(TypeId::of::<ResourceA>())
        );

        // Test we can read the `TypeId` of `ComponentA`
        assert_eq!(
            get_type_id_for_type_path(
                &type_registry,
                "bevy_reflect_utils::shared::tests::ComponentA"
            ),
            Some(TypeId::of::<ComponentA>())
        );
    }

    #[test]
    fn get_type_id_for_type_path_fails() {
        let world = create_world();
        let type_registry = world.resource::<AppTypeRegistry>().read();

        // Test we cannot read the TypeId of `NonReflectResource` because it has not been registered and does not
        // implement reflect
        assert_eq!(
            get_type_id_for_type_path(
                &type_registry,
                "bevy_reflect_utils::shared::tests::NonReflectResource"
            ),
            None
        );

        // Test we cannot read the TypeId of `NonRegisteredComponent` because it has not been registered
        assert_eq!(
            get_type_id_for_type_path(
                &type_registry,
                "bevy_reflect_utils::shared::tests::NonRegisteredComponent"
            ),
            None
        );
    }
}
