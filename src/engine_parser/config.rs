#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parameter{
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub const_param: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub ref_param: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub move_param: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub ptr_param: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_generic: bool,
    pub param_type: i32,
    pub name: String,
    ///c type name
    pub type_str: String,
    ///rust type name
    pub r_type: String,
    pub default_value: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CppApi{    
    pub return_type: i32,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_static: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_const: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_virtual: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_override: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_construstor: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_destructor: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_generic: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub ptr_ret: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub const_ret: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub ref_ret: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub invalid: bool,
    pub class_name: String,
    pub name: String,
    pub parameters: Vec<Parameter>,
    ///result c type name
    pub rc_type: String,
    ///result rust type name
    pub r_type: String,
    pub function_block: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Parameters{
    pub parameters: Vec<Parameter>,
}
///all unreal classes are opaque type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CppClass{
    pub name: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CppEnum{
    pub name: String,
    pub constants: Vec<CppEnumConstant>,
    pub enum_class: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CppEnumConstant{    
    pub name: String,
    pub value: i32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CppProperty{
    pub value_type: i32,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_static: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_const: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_ptr: bool,
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub is_generic: bool,
    ///bit value supporte only for opaque
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub bit_value: bool,
    ///not supported yet
    #[serde(skip_serializing_if = "super::is_false", default)]
    pub unsupported: bool,
    pub name: String,
    ///c type name
    pub type_str: String,
    ///rust type name
    pub r_type: String,    
    pub value: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportDetails{
    pub ExportClasses: Vec<CppClass>,
    pub ExportApis: Vec<Parameters>,
    pub ExportEnums: Vec<CppEnum>,
    pub ExportConsts: Vec<CppProperty>,
}
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CustomSettings{
    pub EngineRoot: String,
    ///导出对象列表
    pub ExportClasses: Vec<String>,
    ///强制性导出不透明对象
    pub ForceOpaque: Vec<String>,
    ///黑名单类型(接口)
    pub BlackList: Vec<String>,
    pub ExportApis: Vec<String>,
    pub ExportEnums: Vec<String>,
    ///忽略文件列表
    pub IgnoreFiles: Vec<String>,
    ///支持的导出目录
    pub ExportPathRoot: Vec<String>,
    ///cpp style type wrapper, key is unreal type, value is the wrapped type(witch should be defined by user)
    pub TypeWrapper: Vec<[String;2]>,
}