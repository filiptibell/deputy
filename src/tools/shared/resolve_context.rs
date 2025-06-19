use async_language_server::lsp_types::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Error as JsonError, Value as JsonValue};

/**
    A partial resolve context.

    This does not contain the inner (custom) context value.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveContextPartial {
    pub uri: Url,
}

impl TryFrom<JsonValue> for ResolveContextPartial {
    type Error = JsonError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl TryFrom<&JsonValue> for ResolveContextPartial {
    type Error = JsonError;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value.clone())
    }
}

impl From<ResolveContextPartial> for JsonValue {
    fn from(value: ResolveContextPartial) -> Self {
        serde_json::to_value(value).unwrap()
    }
}

/**
    A context for a future resolve request.

    Contains an inner, custom context value.
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveContext<T>
where
    T: core::fmt::Debug + Clone + Serialize,
{
    pub uri: Url,
    pub value: T,
}

impl<T> ResolveContext<T>
where
    T: core::fmt::Debug + Clone + Serialize,
{
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> TryFrom<JsonValue> for ResolveContext<T>
where
    T: core::fmt::Debug + Clone + Serialize + DeserializeOwned,
{
    type Error = JsonError;
    fn try_from(value: JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl<T> TryFrom<&JsonValue> for ResolveContext<T>
where
    T: core::fmt::Debug + Clone + Serialize + DeserializeOwned,
{
    type Error = JsonError;
    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        serde_json::from_value(value.clone())
    }
}

impl<T> From<ResolveContext<T>> for JsonValue
where
    T: core::fmt::Debug + Clone + Serialize,
{
    fn from(value: ResolveContext<T>) -> Self {
        serde_json::to_value(value).unwrap()
    }
}
