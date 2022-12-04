#![allow(non_snake_case)]
use serde::Deserialize;
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    I32 = 0,
    U32 = 1,
    U64 = 2,
    I64 = 3,
    Usize = 4,
    F32 = 5,
    F64 = 6,
    I16 = 7,
    U16 = 8,
    U8 = 9,
    I8 = 10,
    Bool = 11,
    Void = 12,
    CStr = 13,
    Ptr = 14,
    FName = 15,
    FString = 16,
    Object = 17,
    Undefined = 25,
}
impl From<i32> for ValueType{
    fn from(v: i32) -> Self {
        match v {
            0 => Self::I32,
            1 => Self::U32,
            2 => Self::U64,
            3 => Self::I64,
            4 => Self::Usize,
            5 => Self::F32,
            6 => Self::F64,
            7 => Self::I16,
            8 => Self::U16,
            9 => Self::U8,
            10 => Self::I8,
            11 => Self::Bool,
            12 => Self::Void,
            13 => Self::CStr,
            14 => Self::Ptr,
            15 => Self::FName,
            16 => Self::FString,
            17 => Self::Object,
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
    pub const_param: bool,
    pub ref_param: bool,
    pub move_param: bool,
    pub ptr_param: bool,
    pub param_type: i32,
    pub name: String,
    ///c type name
    pub type_str: String,
    ///rust type name
    pub r_type: String,
    pub default_value: Option<String>,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CppApi{
    pub return_type: i32,
    pub is_static: bool,
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_construstor: bool,
    pub is_destructor: bool,
    pub ptr_ret: bool,
    pub ret_is_const: bool,
    pub class_name: String,
    pub name: String,
    pub parameters: Vec<Parameter>,
    ///result c type name
    pub rc_type: String,
    ///result rust type name
    pub r_type: String,
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
    pub value_type: i32,
    pub is_static: bool,
    pub is_const: bool,
    pub is_ptr: bool,
    ///not supported yet
    pub unsupported: bool,
    pub name: String,
    ///c type name
    pub type_str: String,
    ///rust type name
    pub r_type: String,    
    pub value: Option<String>,
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
    pub IgnoreFiles: Vec<String>,
}