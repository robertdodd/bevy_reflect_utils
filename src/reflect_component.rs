use std::any::TypeId;

use bevy::{prelude::*, reflect::TypeRegistry};

use crate::*;

/// Read the value of a field from an entity's component cast as the specified type.
pub fn reflect_component_read_path<T: Reflect + Clone>(
    entity_ref: &EntityRef,
    type_registry: &TypeRegistry,
    component_type_id: TypeId,
    path: &str,
) -> Result<T, ReflectError> {
    with_component_reflect_field(
        entity_ref,
        type_registry,
        component_type_id,
        path,
        |field| {
            field
                .downcast_ref::<T>()
                .cloned()
                .ok_or(ReflectError::InvalidDowncast)
        },
    )?
}

/// Utility that reads the value of a field on a resource by path, downcast to the provided type.
pub fn reflect_component_read_path_serialized(
    world: &World,
    entity: Entity,
    component_type_id: TypeId,
    path: &str,
) -> Result<String, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    with_component_reflect_field(
        &entity_ref,
        &type_registry,
        component_type_id,
        path,
        |field| serialize_reflect_value(&type_registry, field),
    )?
}

/// Utility that returns the value of `reflect_partial_eq` against a serialized value on a component.
pub fn reflect_component_partial_eq_serialized(
    world: &mut World,
    entity: Entity,
    component_type_id: TypeId,
    path: &str,
    serialized_value: &str,
) -> Result<bool, ReflectError> {
    // De-serialize the value into a `Box<dyn Reflect>`
    let value = deserialize_reflect_value(world, serialized_value)?;

    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    with_component_reflect_field(
        &entity_ref,
        &type_registry,
        component_type_id,
        path,
        |reflect_field| {
            let is_eq = reflect_field.reflect_partial_eq(value.as_reflect());
            is_eq.ok_or(ReflectError::PartialEq)
        },
    )?
}

/// Utility that sets the value of a field on a resource by path.
pub fn reflect_component_set_path_serialized(
    world: &mut World,
    entity: Entity,
    component_type_id: TypeId,
    path: &str,
    serialized_value: &str,
) -> ReflectSetResult {
    // De-serialize the value into a `Box<dyn Reflect>`
    let value = deserialize_reflect_value(world, serialized_value)?;

    with_reflect_component_field_mut_world(
        world,
        component_type_id,
        entity,
        path,
        |reflect_field| {
            let is_eq = reflect_field.reflect_partial_eq(value.as_reflect());
            match is_eq {
                Some(true) => Ok(ReflectSetSuccess::NoChanges),
                _ => match reflect_field.set(value) {
                    Ok(_) => Ok(ReflectSetSuccess::Changed),
                    // NOTE: The error message contained below is not useful, it is usually the name of the dynamic type,
                    // e.g. "DynamicStruct".
                    Err(_) => Err(ReflectError::SetValueFailed),
                },
            }
        },
    )?
}

/// Utility helper that calls `reflect_read_path` from just the world.
///
/// It saves you from having to pass in an `EntityRef` and `TypeRegistry` if you don't have them already.
pub fn reflect_component_read_path_from_world<T: Reflect + Clone>(
    world: &World,
    entity: Entity,
    component_type_id: TypeId,
    path: &str,
) -> Result<T, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    reflect_component_read_path(&entity_ref, &type_registry, component_type_id, path)
}

/// Set the value of a field by its path on a component on an entity.
pub fn reflect_component_set_path<T: Reflect>(
    world: &mut World,
    component_type_id: TypeId,
    entity: Entity,
    path: &str,
    value: T,
) -> ReflectSetResult {
    with_reflect_component_field_mut_world(
        world,
        component_type_id,
        entity,
        path,
        |reflect_field| {
            let value: Box<dyn Reflect> = Box::new(value);
            let is_eq = reflect_field.reflect_partial_eq(value.as_reflect());
            match is_eq {
                Some(true) => Ok(ReflectSetSuccess::NoChanges),
                _ => match reflect_field.set(value) {
                    Ok(_) => Ok(ReflectSetSuccess::Changed),
                    // NOTE: The error message contained below is not useful, it is usually the name of the dynamic type,
                    // e.g. "DynamicStruct".
                    Err(_) => Err(ReflectError::SetValueFailed),
                },
            }
        },
    )?
}

/// Apply the value of a field by its path on a component on an entity.
///
/// See `Reflect::apply` docs for more information.
pub fn reflect_component_apply_path(
    world: &mut World,
    component_type_id: TypeId,
    entity: Entity,
    path: &str,
    value: &dyn Reflect,
) -> Result<(), ReflectError> {
    with_reflect_component_field_mut_world(world, component_type_id, entity, path, |field| {
        field.apply(value);
        Ok(())
    })?
}

/// Utility that copies the properties of components from one entity to another. Only components that both entities
/// have in common are copied.
///
/// Accepts a `type_id_filter` closure that can be used to select or ignore components by their TypeId.
///
///  ```ignore
/// // Copies component properties from `source_entity` to `target_entity`. Only components that both entities have in
/// // common are copied. The closure tells it to ignore `Transform`, `Parent` and `Children` components.
/// let result = reflect_copy_shared_component_props(
///     world,
///     target_entity,
///     source_entity,
///     // Ignore `Transform`, `Children` and `Parent` components.
///     &|type_id| {
///         type_id != TypeId::of::<Transform>()
///             && type_id != TypeId::of::<Parent>()
///             && type_id != TypeId::of::<Children>()
///     },
/// );
/// ```
///
/// See `ReflectError` docs for more information about the error variants.
pub fn reflect_copy_shared_component_props(
    world: &mut World,
    target_entity: Entity,
    source_entity: Entity,
    type_id_filter: &impl Fn(TypeId) -> bool,
) -> Result<(), ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();

    // Collect a vector of TypeIds for components that both entities have in common, ignoring anywhere
    // `type_id_filter` returns False.
    let component_type_ids: Vec<TypeId> = {
        let source_entity_ref = world
            .get_entity(source_entity)
            .ok_or(ReflectError::EntityNotFound)?;
        let target_entity_ref = world
            .get_entity(target_entity)
            .ok_or(ReflectError::EntityNotFound)?;
        source_entity_ref
            .archetype()
            .components()
            .filter_map(|component_id| {
                world
                    .components()
                    .get_info(component_id)
                    .and_then(|component_info| component_info.type_id())
            })
            // Remove if component is not present on target entity
            .filter(|type_id| target_entity_ref.contains_type_id(*type_id))
            // Remove if type is not reflectable
            .filter(|type_id| type_registry.get(*type_id).is_some())
            // Check against type_id_filter
            .filter(|type_id| type_id_filter(*type_id))
            .collect()
    };

    // Copy components from the source component to the target component, if the target entity contains that component
    for type_id in component_type_ids.iter() {
        let registration = type_registry
            .get(*type_id)
            .ok_or(ReflectError::TypeRegistrationNotFound)?;
        let reflect_component = registration
            .data::<ReflectComponent>()
            .ok_or(ReflectError::TypeRegistrationInvalidCast)?;

        // Clone the value from of the source component
        let source_entity_ref = world
            .get_entity(source_entity)
            .ok_or(ReflectError::EntityNotFound)?;
        let reflect_source = reflect_component
            .reflect(source_entity_ref)
            .ok_or(ReflectError::EntityDoesNotHaveComponent)?;
        let new_value = reflect_source.clone_value();

        // // TODO: Remove debug logging when done
        // error!("COPY COMPONENT: {}", registration.type_info().type_path());
        // if let bevy::reflect::ReflectRef::Struct(data) = new_value.reflect_ref() {
        //     error!("  {:?}", data.as_reflect());
        // };

        // Apply the cloned value to the target entity, if it has the component
        let mut target_entity_ref = world
            .get_entity_mut(target_entity)
            .ok_or(ReflectError::EntityNotFound)?;
        if let Some(mut reflect_target) = reflect_component.reflect_mut(&mut target_entity_ref) {
            reflect_target.apply(new_value.as_reflect());
        }
    }

    Ok(())
}

// NOTE: Keep this around as a reference
// /// Read the value of a field from a `Struct` component on an entity.
// pub fn reflect_read_struct_field<T: Reflect + Clone>(
//     entity_ref: &EntityRef,
//     type_registry: &TypeRegistry,
//     component_type_id: TypeId,
//     field_path: &str,
// ) -> Option<T> {
//     type_registry
//         .get(component_type_id)
//         .and_then(|registration| registration.data::<ReflectComponent>())
//         .and_then(|reflect_component| reflect_component.reflect(*entity_ref))
//         .and_then(|data| match data.reflect_ref() {
//             ReflectRef::Struct(data) => data
//                 .field(field_path)
//                 .and_then(|field| field.downcast_ref::<T>()),
//             _ => None,
//         })
//         .cloned()
// }

/// Runs a closure with the readonly reflected value of a path on an entity's component.
///
/// Returns a `Result` containing the return value of the closure if successful, `ReflectError` otherwise.
///
/// ```ignore
/// let app_type_registry = world.resource::<AppTypeRegistry>().clone();
/// let type_registry = app_type_registry.read();
/// let entity_ref = world.entity(entity);
/// let result: Result<Option<i32>, ReflectError> = with_reflect_field(
///     entity_ref,
///     type_registry,
///     TypeId::of::<MyComponent>,
///     "value",
///     |field| {
///         // Returns `Option<i32>`
///         field.downcast_ref::<i32>().cloned()
///     },
/// );
/// // `result` is `Result<Option<i32>, ReflectError>`
/// match result {
///     Ok(r) => println!("Result of closure: {r:?}"),
///     Err(err) => println!("ReflectError variant: {err}"),
/// }
/// ```
pub fn with_component_reflect_field<T>(
    entity_ref: &EntityRef,
    type_registry: &TypeRegistry,
    component_type_id: TypeId,
    field_path: &str,
    read_fn: impl FnOnce(&dyn Reflect) -> T,
) -> Result<T, ReflectError> {
    let registration = type_registry
        .get(component_type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_component = registration
        .data::<ReflectComponent>()
        .ok_or(ReflectError::TypeRegistrationInvalidCast)?;
    let dyn_reflect = reflect_component
        .reflect(*entity_ref)
        .ok_or(ReflectError::EntityDoesNotHaveComponent)?;
    dyn_reflect
        .reflect_path(field_path)
        .map_err(|err| ReflectError::ReflectPath(err.to_string()))
        .map(read_fn)
}

/// Runs a closure with mutable access to reflected value of a path on an entity's component.
///
/// Returns a `Result` containing the return value of the closure if successful, `ReflectError` otherwise.
///
/// ```ignore
/// // Sets the value of `MyComponent::value` on an entity to `2`.
/// let result: Result<Option<i32>, ReflectError> = with_reflect_field_mut_world(
///     world,
///     TypeId::of::<MyComponent>,
///     entity,
///     "value",
///     |field| {
///         // Returns `bool`
///         reflect_field.set(Box::new(2)).is_ok()
///     },
/// );
/// // `result` is `Result<bool, ReflectError>`
/// match result {
///     Ok(r) => println!("Result of closure: {r:?}"),
///     Err(err) => println!("ReflectError variant: {err}"),
/// }
/// ```
pub fn with_reflect_component_field_mut_world<T>(
    world: &mut World,
    component_type_id: TypeId,
    entity: Entity,
    path: &str,
    update_fn: impl FnOnce(&mut dyn Reflect) -> T,
) -> Result<T, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();

    let registration = type_registry
        .get(component_type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_component = registration
        .data::<ReflectComponent>()
        .ok_or(ReflectError::TypeRegistrationInvalidCast)?;
    let mut entity_mut = world
        .get_entity_mut(entity)
        .ok_or(ReflectError::EntityNotFound)?;
    let mut dyn_reflect = reflect_component
        .reflect_mut(&mut entity_mut)
        .ok_or(ReflectError::EntityDoesNotHaveComponent)?;

    match dyn_reflect.reflect_path_mut(path) {
        Ok(reflect_field) => Ok(update_fn(reflect_field)),
        Err(err) => Err(ReflectError::ReflectPath(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::SystemState;

    use super::*;

    // NOTE: Must derive Reflect to be reflectable
    // NOTE: Must derive Clone, so we can read and return the current value
    #[derive(Reflect, Default, PartialEq, Eq, Clone, Debug, Copy)]
    enum EnumA {
        #[default]
        A,
        B(u32),
    }

    #[derive(Component, Reflect, Default)]
    #[reflect(Component)]
    struct ComponentA {
        value1: EnumA,
        value2: EnumA,
    }

    #[derive(Component, Reflect, Default)]
    #[reflect(Component)]
    struct ComponentB;

    #[derive(Component, Reflect, Default)]
    #[reflect(Component)]
    struct ComponentC(u32);

    #[derive(Component)]
    struct NonReflectComponent;

    /// Test utility that creates a new world and registers the test types
    fn create_world() -> World {
        let mut world = World::new();
        world.init_resource::<AppTypeRegistry>();

        let type_registry = world.resource_mut::<AppTypeRegistry>();
        type_registry.write().register::<ComponentA>();
        type_registry.write().register::<ComponentB>();
        type_registry.write().register::<ComponentC>();

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
    fn reflect_set_path_works() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::B(5),
                    value2: EnumA::A,
                })
                .id()
        });

        // Set the value of `ComponentA::value1` to `EnumA:A`
        reflect_component_set_path(
            &mut world,
            TypeId::of::<ComponentA>(),
            entity,
            "value1",
            EnumA::A,
        )
        .unwrap();

        // Set the value of `ComponentA::value2` to `EnumA:B(2)`
        reflect_component_set_path(
            &mut world,
            TypeId::of::<ComponentA>(),
            entity,
            "value2",
            EnumA::B(2),
        )
        .unwrap();

        // Test the values were set correctly on the component
        let entity_ref = world.entity(entity);
        let component = entity_ref.get::<ComponentA>().unwrap();
        assert_eq!(component.value1, EnumA::A);
        assert_eq!(component.value2, EnumA::B(2));
    }

    #[test]
    fn reflect_set_path_errors() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::B(5),
                    value2: EnumA::A,
                })
                .id()
        });

        // Set the value of `ComponentA::value1` to `EnumA:A`
        let result = reflect_component_set_path(
            &mut world,
            TypeId::of::<ComponentA>(),
            entity,
            "invalid_path",
            EnumA::A,
        );
        assert!(matches!(result, Err(ReflectError::ReflectPath(_))));
    }

    #[test]
    fn reflect_read_path_works() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::A,
                    value2: EnumA::B(2),
                })
                .id()
        });

        let type_registry = world.resource::<AppTypeRegistry>().read();
        let entity_ref = world.entity(entity);

        // Test we can read the value of ComponentA::value1
        let value1 = reflect_component_read_path::<EnumA>(
            &entity_ref,
            &type_registry,
            TypeId::of::<ComponentA>(),
            "value1",
        )
        .unwrap();
        assert_eq!(value1, EnumA::A);

        // Test we can read the value of ComponentA::value2
        let value2 = reflect_component_read_path::<EnumA>(
            &entity_ref,
            &type_registry,
            TypeId::of::<ComponentA>(),
            "value2",
        )
        .unwrap();
        assert_eq!(value2, EnumA::B(2));

        // Test we can read the value of ComponentA::value2.0
        let value2_tuple_value = reflect_component_read_path::<u32>(
            &entity_ref,
            &type_registry,
            TypeId::of::<ComponentA>(),
            "value2.0",
        )
        .unwrap();
        assert_eq!(value2_tuple_value, 2);
    }

    #[test]
    fn reflect_read_path_errors() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::A,
                    value2: EnumA::B(2),
                })
                .id()
        });

        let type_registry = world.resource::<AppTypeRegistry>().read();
        let entity_ref = world.entity(entity);

        #[derive(Component, Reflect, Default)]
        #[reflect(Component)]
        struct UnregisteredComponent {
            value: bool,
        }

        // Test we get the correct error when the component is not registered
        let result = reflect_component_read_path::<bool>(
            &entity_ref,
            &type_registry,
            TypeId::of::<UnregisteredComponent>(),
            "value",
        );
        assert!(matches!(
            result,
            Err(ReflectError::TypeRegistrationNotFound)
        ));

        // Test we get the correct error when the entity does not have the component
        let result = reflect_component_read_path::<bool>(
            &entity_ref,
            &type_registry,
            TypeId::of::<ComponentB>(),
            "value",
        );
        assert!(matches!(
            result,
            Err(ReflectError::EntityDoesNotHaveComponent)
        ));

        // Test we get the correct error when reading an invalid path
        let result = reflect_component_read_path::<bool>(
            &entity_ref,
            &type_registry,
            TypeId::of::<ComponentA>(),
            "not_a_field",
        );
        assert!(matches!(result, Err(ReflectError::ReflectPath(_))));

        // Test we get the correct error when downcasting to the wrong type
        let result = reflect_component_read_path::<f32>(
            &entity_ref,
            &type_registry,
            TypeId::of::<ComponentA>(),
            "value2.0",
        );
        assert!(matches!(result, Err(ReflectError::InvalidDowncast)));
    }

    #[test]
    fn reflect_read_path_from_world_works() {
        let mut world = create_world();
        let entity = run_with_commands(&mut world, |commands| {
            commands
                .spawn(ComponentA {
                    value1: EnumA::A,
                    value2: EnumA::B(2),
                })
                .id()
        });

        // Test we can read the value of ComponentA::value1
        let value1 = reflect_component_read_path_from_world::<EnumA>(
            &world,
            entity,
            TypeId::of::<ComponentA>(),
            "value1",
        )
        .unwrap();
        assert_eq!(value1, EnumA::A);

        // Test we can read the value of ComponentA::value2
        let value2 = reflect_component_read_path_from_world::<EnumA>(
            &world,
            entity,
            TypeId::of::<ComponentA>(),
            "value2",
        )
        .unwrap();
        assert_eq!(value2, EnumA::B(2));

        // Test we can read the value of ComponentA::value2.0
        let value2_tuple_value = reflect_component_read_path_from_world::<u32>(
            &world,
            entity,
            TypeId::of::<ComponentA>(),
            "value2.0",
        )
        .unwrap();
        assert_eq!(value2_tuple_value, 2);
    }

    #[test]
    fn reflect_copy_shared_component_props_works() {
        let source_value1 = EnumA::A;
        let source_value2 = EnumA::B(2);

        let mut world = create_world();
        let (target_entity, source_entity) = run_with_commands(&mut world, |commands| {
            let source_entity = commands
                .spawn((
                    ComponentA {
                        value1: source_value1,
                        value2: source_value2,
                    },
                    ComponentC(1),
                ))
                .id();
            let target_entity = commands
                .spawn((
                    ComponentA {
                        value1: EnumA::B(1),
                        value2: EnumA::A,
                    },
                    ComponentC(2),
                ))
                .id();
            (target_entity, source_entity)
        });

        let result = reflect_copy_shared_component_props(
            &mut world,
            target_entity,
            source_entity,
            &|type_id| type_id != TypeId::of::<ComponentC>(),
        );
        assert!(result.is_ok());

        // test that the source entity is unchanged
        let source_component = world.entity(source_entity).get::<ComponentA>().unwrap();
        assert_eq!(source_component.value1, source_value1);
        assert_eq!(source_component.value2, source_value2);

        // test that target_entity was updated to match source_entity
        let target_component = world.entity(target_entity).get::<ComponentA>().unwrap();
        assert_eq!(target_component.value1, source_value1);
        assert_eq!(target_component.value2, source_value2);

        // test that ComponentC was unchanged, because we excluded it in the type id filter
        let target_component = world.entity(target_entity).get::<ComponentC>().unwrap();
        assert_eq!(target_component.0, 2);
    }

    #[test]
    fn reflect_copy_shared_component_props_works_when_target_does_no_have_entity() {
        let mut world = create_world();
        let (target_entity, source_entity) = run_with_commands(&mut world, |commands| {
            let target_entity = commands.spawn_empty().id();
            let source_entity = commands
                .spawn(ComponentA {
                    value1: EnumA::A,
                    value2: EnumA::A,
                })
                .id();
            (target_entity, source_entity)
        });

        let result =
            reflect_copy_shared_component_props(&mut world, target_entity, source_entity, &|_| {
                true
            });
        if let Err(err) = result {
            panic!("Failed: {err:?}");
        }

        // test that target_entity does not have component A
        assert!(!world.entity(target_entity).contains::<ComponentA>());
    }

    #[test]
    fn reflect_copy_shared_component_props_works_with_non_reflect_components() {
        let mut world = create_world();
        let (target_entity, source_entity) = run_with_commands(&mut world, |commands| {
            let target_entity = commands.spawn(NonReflectComponent).id();
            let source_entity = commands.spawn(NonReflectComponent).id();
            (target_entity, source_entity)
        });

        let result =
            reflect_copy_shared_component_props(&mut world, target_entity, source_entity, &|_| {
                true
            });
        if let Err(err) = result {
            panic!("Failed: {err:?}");
        }

        // test that target_entity does not have component A
        assert!(!world.entity(target_entity).contains::<ComponentA>());
    }
}
