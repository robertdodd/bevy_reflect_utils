use core::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ReflectSetSuccess {
    Changed,
    NoChanges,
}

pub type ReflectSetResult = Result<ReflectSetSuccess, ReflectError>;

/// Error variants for the `bevy_reflect_utils` crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReflectError {
    /// The entity was not found in the world.
    EntityNotFound,
    /// The component type ID was not found in the `TypeRegistry` resource.
    TypeRegistrationNotFound,
    /// The component's type registration data could not be cast to the specified type.
    ///
    /// This error happens if calling `type_registration.data::<ReflectComponent>()` returns `None`.
    TypeRegistrationInvalidCast,
    /// The component type registration could not be reflected from the entity, meaning the entity does not contain
    /// the component.
    EntityDoesNotHaveComponent,
    /// There was an error reflecting the path on the component.
    ReflectPath(String),
    /// The value of a reflected field could not be downcast to the specified type.
    InvalidDowncast,
    /// No default value registered for the type.
    NoDefaultValue,
    /// Setting the value of a reflected value failed.
    SetValueFailed,
    /// The resource does not exist in the world
    ResourceDoesNotExist,
    /// Serialization Failed
    Serialize(String),
    /// De-serialization Failed
    Deserialize(String),
    /// Reflect PartialEq Failed
    PartialEq,
    /// Cannot get access to the resource with the given [`ComponentId`] in the world as it conflicts with an on going operation.
    NoAccess,
}

impl fmt::Display for ReflectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReflectError::EntityNotFound => write!(f, "Entity not found"),
            ReflectError::TypeRegistrationNotFound => write!(f, "Type registration not found"),
            ReflectError::TypeRegistrationInvalidCast => write!(
                f,
                "Type registration could not be cast to the specified type"
            ),
            ReflectError::EntityDoesNotHaveComponent => write!(f, "Entity does not have component"),
            ReflectError::ReflectPath(err) => write!(f, "Reflect path error: {}", err),
            ReflectError::InvalidDowncast => write!(f, "Invalid downcast"),
            ReflectError::NoDefaultValue => {
                write!(f, "No default value registration for the specified type")
            }
            ReflectError::SetValueFailed => write!(f, "Set value failed"),
            ReflectError::ResourceDoesNotExist => write!(f, "Resource does not exist"),
            ReflectError::Serialize(err) => write!(f, "Serialization failed: {err}"),
            ReflectError::Deserialize(err) => write!(f, "De-serialization failed: {err}"),
            ReflectError::PartialEq => write!(f, "Reflect PartialEq failed"),
            ReflectError::NoAccess => write!(f, "No access to resource"),
        }
    }
}
