use std::any::TypeId;

use bevy::{
    prelude::*,
    reflect::{TypeData, TypeRegistry},
};

use crate::ReflectError;

pub fn with_reflect_trait_on_entity_world<T: TypeData, R>(
    world: &mut World,
    entity: Entity,
    type_id: TypeId,
    get_fn: impl FnOnce(&dyn Reflect, &T) -> R,
) -> Result<R, ReflectError> {
    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    with_reflect_trait_on_entity(&type_registry, entity_ref, type_id, get_fn)
}

pub fn with_reflect_trait_on_entity<T: TypeData, R>(
    type_registry: &TypeRegistry,
    entity_ref: EntityRef,
    type_id: TypeId,
    get_fn: impl FnOnce(&dyn Reflect, &T) -> R,
) -> Result<R, ReflectError> {
    let reflect_trait = type_registry
        .get_type_data::<T>(type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_component = type_registry
        .get_type_data::<ReflectComponent>(type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_value = reflect_component
        .reflect(entity_ref)
        .ok_or(ReflectError::EntityDoesNotHaveComponent)?;
    Ok(get_fn(reflect_value, reflect_trait))
}

pub fn with_reflect_trait_on_entity_mut_world<T: TypeData, R>(
    world: &mut World,
    entity: Entity,
    type_id: TypeId,
    get_fn: impl FnOnce(&mut dyn Reflect, &T) -> R,
) -> Result<R, ReflectError> {
    world.resource_scope(|world, app_type_registry: Mut<AppTypeRegistry>| {
        let type_registry = app_type_registry.read();

        let mut entity_mut = world
            .get_entity_mut(entity)
            .ok_or(ReflectError::EntityNotFound)?;

        with_reflect_trait_on_entity_mut(&type_registry, &mut entity_mut, type_id, get_fn)
    })
}

pub fn with_reflect_trait_on_entity_mut<T: TypeData, R>(
    type_registry: &TypeRegistry,
    entity_mut: &mut EntityWorldMut,
    type_id: TypeId,
    get_fn: impl FnOnce(&mut dyn Reflect, &T) -> R,
) -> Result<R, ReflectError> {
    let reflect_trait = type_registry
        .get_type_data::<T>(type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_component = type_registry
        .get_type_data::<ReflectComponent>(type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let mut reflect_value = reflect_component
        .reflect_mut(entity_mut)
        .ok_or(ReflectError::EntityDoesNotHaveComponent)?;
    Ok(get_fn(reflect_value.as_reflect_mut(), reflect_trait))
}

pub fn reflect_find_trait_on_entity<T: TypeData, R>(
    world: &mut World,
    entity: Entity,
    trait_handler: impl Fn(&dyn Reflect, &T) -> Option<R>,
) -> Result<Option<R>, ReflectError> {
    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    // Find the first result that returns `Some`
    let result = entity_ref
        .archetype()
        .components()
        .filter_map(|component_id| {
            world
                .components()
                .get_info(component_id)
                .and_then(|component_info| component_info.type_id())
        })
        .filter_map(|type_id| {
            let reflect_trait = type_registry.get_type_data::<T>(type_id);
            reflect_trait.map(|reflect_trait| (type_id, reflect_trait))
        })
        .filter_map(|(type_id, reflect_trait)| {
            let reflect_component = type_registry.get_type_data::<ReflectComponent>(type_id);
            reflect_component.map(|reflect_component| (reflect_trait, reflect_component))
        })
        .find_map(|(reflect_trait, reflect_component)| {
            reflect_component
                .reflect(entity_ref)
                .and_then(|reflect_value| trait_handler(reflect_value, reflect_trait))
        });

    Ok(result)
}

pub fn reflect_trait_find_one<T: TypeData, R>(
    world: &mut World,
    entity: Entity,
    mut callback: impl FnMut(&dyn Reflect, &T) -> Option<R>,
) -> Result<Option<R>, ReflectError> {
    let mut return_value: Option<R> = None;
    let result = reflect_trait_iter(world, entity, |reflect_value, reflect_trait| {
        return_value = callback(reflect_value, reflect_trait);
        return_value.is_none() // `false` to stop iterating if return value is `Some`, `true` to keep iterating
    });
    result.map(|_| return_value)
}

pub fn reflect_trait_once<T: TypeData>(
    world: &mut World,
    entity: Entity,
    mut callback: impl FnMut(&dyn Reflect, &T),
) -> Result<(), ReflectError> {
    let mut was_called = false;
    let result = reflect_trait_iter(world, entity, |reflect_value, reflect_trait| {
        was_called = true;
        callback(reflect_value, reflect_trait);
        false // stop iterating
    });
    match (was_called, result) {
        (true, Ok(_)) => Ok(()),
        (false, Ok(_)) => Err(ReflectError::EntityDoesNotHaveComponent),
        (_, err) => err,
    }
}

pub fn reflect_trait_mut_expect_once<T: TypeData>(
    world: &mut World,
    entity: Entity,
    mut callback: impl FnMut(&mut dyn Reflect, &T),
) -> Result<(), ReflectError> {
    let mut was_called = false;
    let result = reflect_trait_iter_mut(world, entity, |reflect_value, reflect_trait| {
        was_called = true;
        callback(reflect_value, reflect_trait);
        false
    });
    match (was_called, result) {
        (true, Ok(_)) => Ok(()),
        (false, Ok(_)) => Err(ReflectError::EntityDoesNotHaveComponent),
        (_, err) => err,
    }
}

/// Utility that calls a closure on all components that reflect a trait, with immutable access.
///
/// The closure can return `true` to keep iterating, `false` to stop.
///
/// # Example:
///
/// ```rust,ignore
/// #[reflect_trait]
/// pub trait MyTrait {
///     fn get_name(&self) -> String;
/// }
///
/// let mut name: Option<String> = None;
/// let result = reflect_trait_iter::<ReflectMyTrait>(world, entity, |reflect_value, reflect_trait| {
///     if let Some(my_trait) = reflect_trait.get(reflect_value) {
///         count += 1;
///         let name = my_trait.get_name();
///         println!("name: {}", name);
///         false // `false` to stop iterating, now that we found a match
///     } else {
///         true // `true` to keep iterating
///     }
/// });
/// match result {
///     Ok(()) => info!("Reflect trait success")
///     Err(err) => error!("Error reflecting trait: {err:?}")
/// }
///
/// // The function call will return `Ok` if no matching types are found, so you may want to handle the case when no
/// // types are found yourself:
/// if count == 0 {
///     error!("No components reflecting `MyTrait` found on entity {entity:?}");
/// }
/// ```
pub fn reflect_trait_iter<T: TypeData>(
    world: &mut World,
    entity: Entity,
    mut callback: impl FnMut(&dyn Reflect, &T) -> bool,
) -> Result<(), ReflectError> {
    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    let app_type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = app_type_registry.read();

    // Find the first result that returns `Some`
    entity_ref
        .archetype()
        .components()
        .filter_map(|component_id| {
            world
                .components()
                .get_info(component_id)
                .and_then(|component_info| component_info.type_id())
        })
        .filter_map(|type_id| {
            let reflect_trait = type_registry.get_type_data::<T>(type_id);
            reflect_trait.map(|reflect_trait| (type_id, reflect_trait))
        })
        .filter_map(|(type_id, reflect_trait)| {
            let reflect_component = type_registry.get_type_data::<ReflectComponent>(type_id);
            reflect_component.map(|reflect_component| (reflect_trait, reflect_component))
        })
        .find(|(reflect_trait, reflect_component)| {
            if let Some(reflect_value) = reflect_component.reflect(entity_ref) {
                let must_continue = callback(reflect_value, reflect_trait);
                !must_continue
            } else {
                false
            }
        });

    Ok(())
}

/// Utility that calls a closure on all components that reflect a trait, with mutable access.
///
/// The closure can return `true` to keep iterating, `false` to stop.
///
/// # Example:
///
/// ```rust,ignore
/// #[reflect_trait]
/// pub trait MyTrait {
///     fn set_name(&mut self, new_name: String);
/// }
///
/// let mut count = 0;
/// let result = reflect_trait_iter_mut::<ReflectMyTrait>(world, entity, |reflect_value, reflect_trait| {
///     if let Some(my_trait) = reflect_trait.get_mut(reflect_value) {
///         count += 1;
///         my_trait.set_name("New Name".to_string());
///     }
///     true // return `false` instead to keep iterating
/// });
/// match result {
///     Ok(()) => info!("Successfully updated {} components", count)
///     Err(err) => error!("Error reflecting trait: {err:?}")
/// }
///
/// // The function call will return `Ok` if no matching types are found, so you may want to handle the case when no
/// // types are found yourself:
/// if count == 0 {
///     error!("No components reflecting `MyTrait` found on entity {entity:?}");
/// }
/// ```
pub fn reflect_trait_iter_mut<T: TypeData>(
    world: &mut World,
    entity: Entity,
    mut callback: impl FnMut(&mut dyn Reflect, &T) -> bool,
) -> Result<(), ReflectError> {
    let entity_ref = world
        .get_entity(entity)
        .ok_or(ReflectError::EntityNotFound)?;

    // Collect a vector of component `TypeId`s from the entity.
    // We need to collect them first because we need mutable world access below.
    let type_ids: Vec<TypeId> = entity_ref
        .archetype()
        .components()
        .filter_map(|component_id| {
            world
                .components()
                .get_info(component_id)
                .and_then(|component_info| component_info.type_id())
        })
        .collect();

    world.resource_scope(
        |world, app_type_registry: Mut<AppTypeRegistry>| -> Result<(), ReflectError> {
            let type_registry = app_type_registry.read();

            // filter and map the type ids to reflection types
            let reflect_iter = type_ids.iter().filter_map(|type_id| {
                let reflect_trait = type_registry.get_type_data::<T>(*type_id);
                reflect_trait.and_then(|reflect_trait| {
                    let reflect_component =
                        type_registry.get_type_data::<ReflectComponent>(*type_id);
                    reflect_component.map(|reflect_component| (reflect_trait, reflect_component))
                })
            });

            // run the callback for each component we can reflect the trait for
            for (reflect_trait, reflect_component) in reflect_iter {
                let mut entity_mut = world
                    .get_entity_mut(entity)
                    .ok_or(ReflectError::EntityNotFound)?;
                if let Some(mut reflect_value) = reflect_component.reflect_mut(&mut entity_mut) {
                    // Call `callback`. Break the loop if it returns `true`.
                    let must_continue = callback(reflect_value.as_reflect_mut(), reflect_trait);
                    if !must_continue {
                        break;
                    }
                }
            }
            Ok(())
        },
    )
}
