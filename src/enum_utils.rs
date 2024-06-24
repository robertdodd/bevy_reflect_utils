use std::any::TypeId;

use bevy::{
    prelude::*,
    reflect::{DynamicEnum, Enum, ReflectRef, TypeRegistry},
};

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EnumDirection {
    Forward,
    Backward,
}

pub enum NextEnumVariant {
    Ok(DynamicEnum),
    NoChanges,
}

/// Utility helper that calls `reflect_component_read_enum_variant_name` from just the world.
///
/// It saves you from having to pass in an `EntityRef` and `TypeRegistry` if you don't have them already.
pub fn reflect_component_read_enum_variant_name_from_world(
    world: &World,
    entity: Entity,
    component_type_id: TypeId,
    path: &str,
) -> Result<String, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    if let Some(entity_ref) = world.get_entity(entity) {
        reflect_component_read_enum_variant_name(
            &entity_ref,
            &type_registry,
            component_type_id,
            path,
        )
    } else {
        Err(ReflectError::EntityNotFound)
    }
}

/// Reads the name of the enum variant set on a path on a component on an entity.
pub fn reflect_component_read_enum_variant_name(
    entity_ref: &EntityRef,
    type_registry: &TypeRegistry,
    component_type_id: TypeId,
    field_path: &str,
) -> Result<String, ReflectError> {
    with_component_reflect_field(
        entity_ref,
        type_registry,
        component_type_id,
        field_path,
        |field| match field.reflect_ref() {
            ReflectRef::Enum(dyn_enum) => Ok(dyn_enum.variant_name().to_string()),
            _ => Err(ReflectError::InvalidDowncast),
        },
    )?
}

/// Reads the name of the enum variant set on a path on a component on an entity.
pub fn reflect_resource_read_enum_variant_name(
    world: &World,
    resource_type_id: TypeId,
    field_path: &str,
) -> Result<String, ReflectError> {
    with_resource_reflect_field(world, resource_type_id, field_path, |field| {
        match field.reflect_ref() {
            ReflectRef::Enum(dyn_enum) => Ok(dyn_enum.variant_name().to_string()),
            _ => Err(ReflectError::InvalidDowncast),
        }
    })?
}

/// Apply the value of a field by its path on a component on an entity.
///
/// Returns:
/// - Ok(true) - if successful and the field was changed.
/// - Ok(false) - if successful, but the field was not changed.
/// - Err(String) - if there was an error reading or updating the field.
///
/// See `Reflect::apply` docs for more information.
pub fn reflect_component_toggle_enum_variant(
    world: &mut World,
    component_type_id: TypeId,
    entity: Entity,
    path: &str,
    direction: EnumDirection,
    wrap: bool,
) -> ReflectSetResult {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();

    with_reflect_component_field_mut_world(world, component_type_id, entity, path, |field| {
        if let ReflectRef::Enum(dyn_enum) = field.reflect_ref() {
            let next_variant = get_next_enum_variant(dyn_enum, &type_registry, direction, wrap)?;
            match next_variant {
                NextEnumVariant::Ok(next_value) => {
                    field.apply(next_value.as_reflect());
                    Ok(ReflectSetSuccess::Changed)
                }
                NextEnumVariant::NoChanges => Ok(ReflectSetSuccess::NoChanges),
            }
        } else {
            Err(ReflectError::InvalidDowncast)
        }
    })?
}

/// Apply the value of a field by its path on a component on an entity.
///
/// Returns:
/// - Ok(true) - if successful and the field was changed.
/// - Ok(false) - if successful, but the field was not changed.
/// - Err(String) - if there was an error reading or updating the field.
///
/// See `Reflect::apply` docs for more information.
pub fn reflect_resource_toggle_enum_variant(
    world: &mut World,
    resource_type_id: TypeId,
    path: &str,
    direction: EnumDirection,
    wrap: bool,
) -> ReflectSetResult {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();

    with_resource_reflect_field_mut(world, resource_type_id, path, |field| {
        if let ReflectRef::Enum(dyn_enum) = field.reflect_ref() {
            let next_variant = get_next_enum_variant(dyn_enum, &type_registry, direction, wrap)?;
            match next_variant {
                NextEnumVariant::Ok(next_value) => {
                    field.apply(next_value.as_reflect());
                    Ok(ReflectSetSuccess::Changed)
                }
                NextEnumVariant::NoChanges => Ok(ReflectSetSuccess::NoChanges),
            }
        } else {
            Err(ReflectError::InvalidDowncast)
        }
    })?
}

/// Utility that returns the next index in a range in a specified direction, with optional "wrap-around" functionality
/// via the `wrap` argument.
///
/// NOTE: If `Some(new_index)` is returned, `new_index` is guaranteed to be different to the current index.
fn get_next_index_in_direction(
    index: usize,
    length: usize,
    direction: EnumDirection,
    wrap: bool,
) -> Option<usize> {
    if length == 1 || index > length - 1 {
        return None;
    }
    match direction {
        EnumDirection::Forward => {
            if index < length - 1 {
                Some(index + 1)
            } else if wrap {
                Some(0)
            } else {
                None
            }
        }
        EnumDirection::Backward => {
            if index > 0 {
                Some(index - 1)
            } else if wrap {
                Some(length - 1)
            } else {
                None
            }
        }
    }
}

/// Read the value at a path on a component on an entity.
pub fn get_next_enum_variant(
    dyn_enum: &dyn Enum,
    type_registry: &TypeRegistry,
    direction: EnumDirection,
    wrap: bool,
) -> Result<NextEnumVariant, ReflectError> {
    let index = dyn_enum.variant_index();
    let type_info = dyn_enum.get_represented_type_info().unwrap();

    if let bevy::reflect::TypeInfo::Enum(enum_info) = type_info {
        // Get the next enum variant in the specified direction.
        // NOTE: `new_variant` will be `None` if the result is unchanged, wrapping is disabled, and it's at the end of
        // the list, or the enum does not contain a variant at the next index. The last should never happen.
        let n_variants = enum_info.iter().count();
        let new_variant = get_next_index_in_direction(index, n_variants, direction, wrap)
            .and_then(|new_index| enum_info.variant_at(new_index));

        match new_variant {
            Some(new_variant_info) => {
                match construct_default_enum_variant(new_variant_info, type_registry) {
                    Ok(result) => Ok(NextEnumVariant::Ok(result)),
                    Err(err) => Err(err),
                }
            }
            None => Ok(NextEnumVariant::NoChanges),
        }
    } else {
        Err(ReflectError::InvalidDowncast)
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemState;

    use super::*;

    #[derive(Reflect, Default, PartialEq, Eq, Debug)]
    enum EnumA {
        #[default]
        A,
        B(u32),
    }

    #[derive(Component, Reflect, Default, Debug)]
    #[reflect(Component)]
    struct ComponentA {
        value1: EnumA,
        value2: EnumA,
    }

    #[derive(Resource, Reflect, Default, Debug)]
    #[reflect(Resource)]
    struct ResourceA {
        value1: EnumA,
        value2: EnumA,
    }

    /// Test utility that creates a new world and registers the test types
    fn create_world() -> World {
        let mut world = World::new();
        world.init_resource::<AppTypeRegistry>();

        let type_registry = world.resource_mut::<AppTypeRegistry>();
        type_registry.write().register::<ComponentA>();
        type_registry.write().register::<ResourceA>();
        type_registry.write().register::<EnumA>();

        world
    }

    /// Test utility that runs commands on a world and returns the result from the closure.
    fn run_with_commands<T>(world: &mut World, cmds: impl FnOnce(&mut Commands) -> T) -> T {
        let mut system_state: SystemState<Commands> = SystemState::new(world);
        let mut commands = system_state.get_mut(world);

        let result = cmds(&mut commands);

        system_state.apply(world);

        result
    }

    #[test]
    fn reflect_component_read_enum_variant_name_works() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::A,
                    value2: EnumA::B(1),
                })
                .id()
        });

        // Test the variant name for each value
        let entity_ref = world.entity(entity);
        let type_id = TypeId::of::<ComponentA>();
        let type_registry = world.resource::<AppTypeRegistry>().read();
        assert_eq!(
            reflect_component_read_enum_variant_name(
                &entity_ref,
                &type_registry,
                type_id,
                "value1"
            )
            .unwrap(),
            "A".to_string()
        );
        assert_eq!(
            reflect_component_read_enum_variant_name(
                &entity_ref,
                &type_registry,
                type_id,
                "value2"
            )
            .unwrap(),
            "B".to_string()
        );
    }

    #[test]
    fn reflect_component_toggle_enum_variant_works() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::A,
                    value2: EnumA::B(1),
                })
                .id()
        });

        // Test toggling ComponentA::value1
        let result = reflect_component_toggle_enum_variant(
            &mut world,
            TypeId::of::<ComponentA>(),
            entity,
            "value1",
            EnumDirection::Forward,
            false,
        );
        assert_eq!(result, Ok(ReflectSetSuccess::Changed));
        assert_eq!(
            world.entity(entity).get::<ComponentA>().unwrap().value1,
            EnumA::B(0)
        );

        // Test toggling ComponentA::value1 without wrapping.
        // The value should be unchanged because EnumA::B is the last variant, and we passed `wrap=false`
        let result = reflect_component_toggle_enum_variant(
            &mut world,
            TypeId::of::<ComponentA>(),
            entity,
            "value1",
            EnumDirection::Forward,
            false,
        );
        assert_eq!(result, Ok(ReflectSetSuccess::NoChanges));
        assert_eq!(
            world.entity(entity).get::<ComponentA>().unwrap().value1,
            EnumA::B(0)
        );

        // Test toggling ComponentA::value1 with wrapping.
        let result = reflect_component_toggle_enum_variant(
            &mut world,
            TypeId::of::<ComponentA>(),
            entity,
            "value1",
            EnumDirection::Forward,
            true,
        );
        assert_eq!(result, Ok(ReflectSetSuccess::Changed));
        assert_eq!(
            world.entity(entity).get::<ComponentA>().unwrap().value1,
            EnumA::A
        );
    }

    #[test]
    fn reflect_resource_toggle_enum_variant_works() {
        let mut world = create_world();
        world.insert_resource(ResourceA {
            value1: EnumA::A,
            value2: EnumA::B(1),
        });

        // Test toggling ResourceA::value1
        let result = reflect_resource_toggle_enum_variant(
            &mut world,
            TypeId::of::<ResourceA>(),
            "value1",
            EnumDirection::Forward,
            false,
        );
        assert_eq!(result, Ok(ReflectSetSuccess::Changed));
        assert_eq!(world.resource::<ResourceA>().value1, EnumA::B(0));

        // Test toggling ResourceA::value1 without wrapping.
        // The value should be unchanged because EnumA::B is the last variant, and we passed `wrap=false`
        let result = reflect_resource_toggle_enum_variant(
            &mut world,
            TypeId::of::<ResourceA>(),
            "value1",
            EnumDirection::Forward,
            false,
        );
        assert_eq!(result, Ok(ReflectSetSuccess::NoChanges));
        assert_eq!(world.resource::<ResourceA>().value1, EnumA::B(0));

        // Test toggling ResourceA::value1 with wrapping.
        let result = reflect_resource_toggle_enum_variant(
            &mut world,
            TypeId::of::<ResourceA>(),
            "value1",
            EnumDirection::Forward,
            true,
        );
        assert_eq!(result, Ok(ReflectSetSuccess::Changed));
        assert_eq!(world.resource::<ResourceA>().value1, EnumA::A);
    }

    #[test]
    fn reflect_resource_read_enum_variant_name_works() {
        let mut world = create_world();
        world.insert_resource(ResourceA {
            value1: EnumA::A,
            value2: EnumA::B(1),
        });

        // Test the variant name for each value
        let type_id = TypeId::of::<ResourceA>();
        // let type_registry = world.resource::<AppTypeRegistry>().read();
        assert_eq!(
            reflect_resource_read_enum_variant_name(&world, type_id, "value1").unwrap(),
            "A".to_string()
        );
        assert_eq!(
            reflect_resource_read_enum_variant_name(&world, type_id, "value2").unwrap(),
            "B".to_string()
        );
    }

    #[test]
    fn get_next_index_in_direction_works() {
        // Test valid movement with wrapping disabled
        assert_eq!(
            get_next_index_in_direction(0, 2, EnumDirection::Forward, false),
            Some(1),
        );
        assert_eq!(
            get_next_index_in_direction(1, 2, EnumDirection::Backward, false),
            Some(0),
        );

        // Test invalid index
        assert_eq!(
            get_next_index_in_direction(5, 2, EnumDirection::Forward, true),
            None,
        );
        assert_eq!(
            get_next_index_in_direction(5, 2, EnumDirection::Forward, false),
            None,
        );
        assert_eq!(
            get_next_index_in_direction(5, 2, EnumDirection::Backward, true),
            None,
        );
        assert_eq!(
            get_next_index_in_direction(5, 2, EnumDirection::Backward, false),
            None,
        );

        // Test end-of-list with wrapping disabled
        assert_eq!(
            get_next_index_in_direction(1, 2, EnumDirection::Forward, false),
            None,
        );
        assert_eq!(
            get_next_index_in_direction(0, 2, EnumDirection::Backward, false),
            None,
        );

        // Test wrapping in multi-item list
        assert_eq!(
            get_next_index_in_direction(1, 2, EnumDirection::Forward, true),
            Some(0),
        );
        assert_eq!(
            get_next_index_in_direction(0, 2, EnumDirection::Backward, true),
            Some(1),
        );

        // Test wrapping disabled with single item list
        assert_eq!(
            get_next_index_in_direction(0, 1, EnumDirection::Forward, false),
            None,
        );
        assert_eq!(
            get_next_index_in_direction(0, 1, EnumDirection::Backward, false),
            None,
        );

        // Test wrapping enabled with single item list
        assert_eq!(
            get_next_index_in_direction(0, 1, EnumDirection::Forward, true),
            None,
        );
        assert_eq!(
            get_next_index_in_direction(0, 1, EnumDirection::Backward, true),
            None,
        );
    }
}
