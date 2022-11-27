use std::collections::BTreeMap;
use super::config::*;
#[derive(Debug, Clone, Default)]
pub struct UnrealClass{    
    pub name: String,
    pub inherit: String,
    pub path: String,
    pub constructors: Vec<Parameters>,
    pub properties: Vec<CppProperty>,
    pub public_apis: Vec<CppApi>,
}
#[derive(Default)]
pub struct Engine{
    pub files: Vec<String>,
    pub file_caches: BTreeMap<String, String>,
    pub static_apis: Vec<CppApi>,
    pub classes: Vec<UnrealClass>
}