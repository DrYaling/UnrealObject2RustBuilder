use std::collections::BTreeMap;
use super::config::*;
#[derive(Debug, Clone, Default)]
pub struct UnrealClass{    
    pub name: String,
    pub inherit: String,
    pub path: String,
    pub is_struct: bool,
    pub opaque: bool,
    pub constructors: Vec<Parameters>,
    pub properties: Vec<CppProperty>,
    pub public_apis: Vec<CppApi>,
}
#[derive(Default)]
pub struct Engine{
    pub files: Vec<String>,
    pub file_caches: BTreeMap<String, String>,
    pub static_apis: Vec<CppApi>,
    pub classes: Vec<UnrealClass>,
    pub enums: Vec<CppEnum>,
    pub value_types: Vec<CppEnum>,
}
unsafe impl Send for Engine{}
unsafe impl Sync for Engine{}