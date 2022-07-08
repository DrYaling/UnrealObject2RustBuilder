use super::property::Property;
use super::api::Api;
use serde::{Deserialize};
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UnrealObject{
    pub unreal_type: String,
    pub r#type: String,
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub opaque_type: bool,
    #[serde(default)]
    pub properties: Vec<Property>,
    #[serde(default)]
    pub apis: Vec<Api>,
}
