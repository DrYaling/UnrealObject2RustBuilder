use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

use super::config::*;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnrealClass{    
    pub name: String,
    pub inherit: String,
    pub path: String,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_struct: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub opaque: bool,
    pub properties: Vec<CppProperty>,
    pub none_public_properties: Vec<CppProperty>,
    pub public_apis: Vec<CppApi>,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Engine{
    #[serde(skip_serializing, default)]
    pub files: Vec<String>,
    #[serde(skip_serializing, default)]
    pub file_paths: BTreeMap<String, String>,
    pub static_apis: Vec<CppApi>,
    pub classes: Vec<UnrealClass>,
    pub enums: Vec<CppEnum>,
    pub value_types: Vec<CppEnum>,
}
unsafe impl Send for Engine{}
unsafe impl Sync for Engine{}