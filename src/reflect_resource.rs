use std::any::TypeId;

use bevy::{prelude::*, reflect::serde::ReflectSerializer, scene::ron};

use crate::*;

/// Utility that reads the value of a field on a resource by path, downcast to the provided type.
pub fn reflect_resource_read_path<T: Reflect + Clone>(
    world: &World,
    resource_type_id: TypeId,
    path: &str,
) -> Result<T, ReflectError> {
    with_resource_reflect_field(world, resource_type_id, path, |field| {
        field
            .downcast_ref::<T>()
            .cloned()
            .ok_or(ReflectError::InvalidDowncast)
    })?
}

/// Utility that reads the value of a field on a resource by path, downcast to the provided type.
pub fn reflect_resource_read_path_serialized(
    world: &World,
    resource_type_id: TypeId,
    path: &str,
) -> Result<String, ReflectError> {
    with_resource_reflect_field(world, resource_type_id, path, |field| {
        let app_type_registry = world.resource::<AppTypeRegistry>();
        let type_registry = app_type_registry.read();

        let serializer = ReflectSerializer::new(field, &type_registry);
        let ron_string = ron::ser::to_string_pretty(&serializer, ron::ser::PrettyConfig::default())
            .map_err(|err| ReflectError::Serialize(format!("{err:?}")))?;
        Ok(ron_string)
    })?
}

/// Utility that sets the value of a field on a resource by path.
pub fn reflect_resource_set_path<T: Reflect>(
    world: &mut World,
    resource_type_id: TypeId,
    path: &str,
    value: T,
) -> ReflectSetResult {
    with_resource_reflect_field_mut(world, resource_type_id, path, |reflect_field| {
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
    })?
}

/// Utility that sets the value of a field on a resource by path.
pub fn reflect_resource_set_path_serialized(
    world: &mut World,
    resource_type_id: TypeId,
    path: &str,
    serialized_value: &str,
) -> ReflectSetResult {
    // De-serialize the value into a `Box<dyn Reflect>`
    let value = deserialize_reflect_value(world, serialized_value)?;

    with_resource_reflect_field_mut(world, resource_type_id, path, |reflect_field| {
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
    })?
}

/// Utility that returns the value of `reflect_partial_eq` against a serialized value.
pub fn reflect_resource_partial_eq_serialized(
    world: &mut World,
    resource_type_id: TypeId,
    path: &str,
    serialized_value: &str,
) -> Result<bool, ReflectError> {
    // De-serialize the value into a `Box<dyn Reflect>`
    let value = deserialize_reflect_value(world, serialized_value)?;

    with_resource_reflect_field(world, resource_type_id, path, |reflect_field| {
        let is_eq = reflect_field.reflect_partial_eq(value.as_reflect());
        is_eq.ok_or(ReflectError::PartialEq)
    })?
}

/// Runs a closure with readonly access to a reflected resource.
///
/// Returns a `Result` containing the return value of the closure if successful, `ReflectError` otherwise.
pub fn with_resource_reflect<T>(
    world: &World,
    resource_type_id: TypeId,
    read_fn: impl FnOnce(&dyn Reflect) -> T,
) -> Result<T, ReflectError> {
    let type_registry = world.resource::<AppTypeRegistry>().read();

    let registration = type_registry
        .get(resource_type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_resource = registration
        .data::<ReflectResource>()
        .ok_or(ReflectError::TypeRegistrationInvalidCast)?;
    reflect_resource
        .reflect(world)
        .ok_or(ReflectError::ResourceDoesNotExist)
        .map(read_fn)
}

/// Runs a closure with mutable access to a reflected resource.
///
/// Returns a `Result` containing the return value of the closure if successful, `ReflectError` otherwise.
pub fn with_resource_reflect_mut<T>(
    world: &mut World,
    resource_type_id: TypeId,
    update_fn: impl FnOnce(Mut<dyn Reflect>) -> T,
) -> Result<T, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();

    let registration = type_registry
        .get(resource_type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_resource = registration
        .data::<ReflectResource>()
        .ok_or(ReflectError::TypeRegistrationInvalidCast)?;
    reflect_resource
        .reflect_mut(world)
        .ok_or(ReflectError::ResourceDoesNotExist)
        .map(update_fn)
}

/// Runs a closure with the mutable reflected value of a path on a resource.
///
/// Returns a `Result` containing the return value of the closure if successful, `ReflectError` otherwise.
///
/// ```ignore
/// let result: Result<bool, ReflectError> = with_resource_reflect_field_mut(
///     &world,
///     TypeId::of::<MyResource>,
///     "value",
///     |field| {
///         // Sets the value of "MyResource::value" to `2`, and returns a `bool` of whether it was successful
///         reflect_field.set(Box::new(2)).is_ok()
///     },
/// );
/// // `result` is `Result<bool, ReflectError>`, because we returned a bool from the closure.
/// match result {
///     Ok(r) => println!("Result of closure: {r:?}"),
///     Err(err) => println!("ReflectError variant: {err}"),
/// }
/// ```
pub fn with_resource_reflect_field_mut<T>(
    world: &mut World,
    resource_type_id: TypeId,
    path: &str,
    update_fn: impl FnOnce(&mut dyn Reflect) -> T,
) -> Result<T, ReflectError> {
    let app_type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = app_type_registry.read();

    let registration = type_registry
        .get(resource_type_id)
        .ok_or(ReflectError::TypeRegistrationNotFound)?;
    let reflect_resource = registration
        .data::<ReflectResource>()
        .ok_or(ReflectError::TypeRegistrationInvalidCast)?;
    let mut dyn_reflect = reflect_resource
        .reflect_mut(world)
        .ok_or(ReflectError::ResourceDoesNotExist)?;
    dyn_reflect
        .reflect_path_mut(path)
        .map_err(|err| ReflectError::ReflectPath(err.to_string()))
        .map(update_fn)
}

/// Runs a closure with the readonly reflected value of a path on a resource.
///
/// Returns a `Result` containing the return value of the closure if successful, `ReflectError` otherwise.
///
/// ```ignore
/// let result: Result<Option<i32>, ReflectError> = with_resource_reflect_field(
///     &world,
///     TypeId::of::<MyResource>,
///     "value",
///     |field| {
///         // Returns `Option<i32>`
///         field.downcast_ref::<i32>().cloned()
///     },
/// );
/// // `result` is `Result<Option<i32>, ReflectError>`, because we return `Option<i32>` from the closure.
/// match result {
///     Ok(r) => println!("Result of closure: {r:?}"),
///     Err(err) => println!("ReflectError variant: {err}"),
/// }
/// ```
pub fn with_resource_reflect_field<T>(
    world: &World,
    resource_type_id: TypeId,
    path: &str,
    read_fn: impl FnOnce(&dyn Reflect) -> T,
) -> Result<T, ReflectError> {
    with_resource_reflect(world, resource_type_id, |dyn_reflect| {
        dyn_reflect
            .reflect_path(path)
            .map_err(|err| ReflectError::ReflectPath(err.to_string()))
            .map(read_fn)
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: Must derive Reflect to be reflectable
    // NOTE: Must derive Clone, so we can read and return the current value
    #[derive(Reflect, Default, PartialEq, Eq, Clone, Debug, Copy)]
    enum EnumA {
        #[default]
        A,
        B(u32),
    }

    #[derive(Resource, Reflect, Default, Clone)]
    #[reflect(Resource)]
    struct ResourceA {
        value1: EnumA,
        value2: EnumA,
    }

    #[derive(Resource, Reflect, Default, Clone)]
    #[reflect(Resource)]
    struct ResourceB;

    #[derive(Resource, Reflect, Default, Clone)]
    #[reflect(Resource)]
    struct ResourceC(u32);

    #[derive(Resource)]
    struct NonReflectResource;

    /// Test utility that creates a new world and registers the test types
    fn create_world() -> World {
        let mut world = World::new();
        world.init_resource::<AppTypeRegistry>();

        let type_registry = world.resource_mut::<AppTypeRegistry>();
        type_registry.write().register::<ResourceA>();
        type_registry.write().register::<ResourceB>();
        type_registry.write().register::<ResourceC>();

        world
    }

    #[test]
    fn reflect_resource_read_path_works() {
        let mut world = create_world();

        let resource_a = ResourceA {
            value1: EnumA::A,
            value2: EnumA::B(1),
        };
        world.insert_resource(resource_a.clone());

        let resource_c = ResourceC(2);
        world.insert_resource(resource_c.clone());

        // Test we can read the value of ResourceA::value1
        let value1 =
            reflect_resource_read_path::<EnumA>(&world, TypeId::of::<ResourceA>(), "value1")
                .unwrap();
        assert_eq!(value1, resource_a.value1);

        // Test we can read the value of ResourceA::value2
        let value2 =
            reflect_resource_read_path::<EnumA>(&world, TypeId::of::<ResourceA>(), "value2")
                .unwrap();
        assert_eq!(value2, resource_a.value2);

        // Test we can read the value of ResourceA::value2
        let resource_c_value =
            reflect_resource_read_path::<u32>(&world, TypeId::of::<ResourceC>(), "0").unwrap();
        assert_eq!(resource_c_value, resource_c.0);
    }

    #[test]
    fn reflect_resource_read_path_errors() {
        let mut world = create_world();

        // Test the error when the resource does not exist
        let result =
            reflect_resource_read_path::<EnumA>(&world, TypeId::of::<ResourceA>(), "value1");
        assert!(matches!(result, Err(ReflectError::ResourceDoesNotExist)));

        world.insert_resource(ResourceA {
            value1: EnumA::A,
            value2: EnumA::B(1),
        });
        world.insert_resource(ResourceB);
        world.insert_resource(ResourceC(2));

        // Test the error when the field does not exist
        let result =
            reflect_resource_read_path::<EnumA>(&world, TypeId::of::<ResourceA>(), "not-a-field");
        assert!(matches!(result, Err(ReflectError::ReflectPath(_))));

        // Test the error when the field is the wrong type
        let result = reflect_resource_read_path::<u32>(&world, TypeId::of::<ResourceA>(), "value1");
        assert!(matches!(result, Err(ReflectError::InvalidDowncast)));

        // Test the error when the type is not registered
        let result =
            reflect_resource_read_path::<u32>(&world, TypeId::of::<NonReflectResource>(), "value1");
        assert!(matches!(
            result,
            Err(ReflectError::TypeRegistrationNotFound)
        ));
    }

    #[test]
    fn reflect_resource_set_path_works() {
        let mut world = create_world();

        let original_resource_a = ResourceA {
            value1: EnumA::A,
            value2: EnumA::B(1),
        };
        world.insert_resource(original_resource_a.clone());

        // Test we can set the value of ResourceA::value1
        let new_value1 = EnumA::B(5);
        let result = reflect_resource_set_path::<EnumA>(
            &mut world,
            TypeId::of::<ResourceA>(),
            "value1",
            new_value1,
        )
        .unwrap();
        assert!(matches!(result, ReflectSetSuccess::Changed));
        assert_eq!(world.resource::<ResourceA>().value1, new_value1);

        // Test we can set the value of ResourceA::value2, but it returns NoChanges when we set the same value.
        let result = reflect_resource_set_path::<EnumA>(
            &mut world,
            TypeId::of::<ResourceA>(),
            "value2",
            original_resource_a.value2,
        )
        .unwrap();
        assert!(matches!(result, ReflectSetSuccess::NoChanges));
        assert_eq!(
            world.resource::<ResourceA>().value2,
            original_resource_a.value2
        );
    }
}
