#![allow(non_snake_case)]
use serde::Deserialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Null = 0,
    I32 = 1,
    I64 = 2,
    U32 = 3,
    U64 = 4,
    Bool = 5,
    SharedPtr = 6,
    Ptr = 7,
    CStr = 8,
    Usize = 9,
    ObjectRef = 10,
    Undefined = 11,
}
impl From<i32> for ValueType{
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Null,
            1 => Self::I32,
            2 => Self::I64,
            3 => Self::U32,
            4 => Self::U64,
            5 => Self::Bool,
            6 => Self::SharedPtr,
            7 => Self::Ptr,
            8 => Self::CStr,
            9 => Self::Usize,
            10 => Self::ObjectRef,
            _ => Self::Undefined,
        }
    }
}
impl PartialEq<i32> for ValueType{
    fn eq(&self, other: &i32) -> bool {
        *self == Self::from(*other)
    }
}
impl PartialEq<ValueType> for i32{
    fn eq(&self, other: &ValueType) -> bool {
        ValueType::from(*self) == *other
    }
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Parameter{
    pub name: String,
    pub const_param: bool,
    pub param_type: i32,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CppApi{
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub r_type: String,
    pub return_type: i32,
    pub function_block: Option<String>,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Parameters{
    pub parameters: Vec<Parameter>,
}
///all unreal classes are opaque type
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CppClass{
    pub name: String,
    pub inherit: String,
    pub path: String,
    pub is_struct: bool,
    pub constructors: Vec<Parameters>,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CppEnum{
    pub name: String,
    pub value: i32,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CppProperty{
    pub name: String,
    pub value_type: i32,
    pub value: String,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ExportDetails{
    pub ExportClasses: Vec<CppClass>,
    pub ExportApis: Vec<Parameters>,
    pub ExportEnums: Vec<CppEnum>,
    pub ExportConsts: Vec<CppProperty>,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CustomSettings{
    pub EngineRoot: String,
    pub ExportClasses: Vec<String>,
    pub ExportApis: Vec<String>,
    pub ExportEnums: Vec<String>,
}