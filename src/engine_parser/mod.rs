use self::config::{Parameter, CppApi};


mod unreal_engine;
pub mod class_parser;
mod config;
// mod r#enum;
mod bindgen;
mod binder_rs;
mod ast;
pub use config::ValueType;
///string not supported yet
fn get_engine_str(type_str: &str) -> Option<(String, ValueType)>{
    match type_str {
        "FName" => Some(("String".to_string(), ValueType::FName)),
        "FText" => Some(("String".to_string(), ValueType::FText)),
        "FString" => Some(("String".to_string(), ValueType::FString)),
        "char*" => Some(("String".to_string(), ValueType::CStr)),
        "const char*" => Some(("String".to_string(), ValueType::CStr)),
        _ => None
    }
}
fn get_object_type(type_str: &str) -> (String, ValueType){    
    (format!("{}", type_str), ValueType::Object)
}
fn parse_c_type(type_str: &str) -> (String, ValueType){
    crate::get_c2r_types(type_str).unwrap_or_else(||{
        get_engine_str(type_str)
        .unwrap_or(
            get_object_type(type_str)
        )
    })
}
fn parse_parameter(parameters: &Vec<Vec<String>>) -> Vec<Parameter>{
    let mut list = vec![];
    let default_flag: String = String::from("=");
    let mut arg_index = 0;
    for param in parameters {
        if param.is_empty() || 
            (param.len() == 1 && param[0].as_str() == "void"){ //void parameter
            continue;
        }
        // let param_len = param.len();
        let mut parameter = Parameter::default();
        let default_value = param.contains(&default_flag);
        let mut iter = param.iter().rev().filter(|s| !s.is_empty());
        if default_value{
            parameter.default_value = iter.next().cloned();
            iter.next();
        }
        // //parameter with no name
        // let maybe_name = if param_len > 1{
        //     iter.next().cloned().expect(&format!("unknown parameter style {:?}", param));
        // }
        // else{
        //     format!("arg{}", arg_index)
        // };
        while let Some(next) =  iter.next(){            
            match next.as_str() {
                "const" => {
                    parameter.const_param = true;
                },
                "const&" => {
                    parameter.const_param = true;
                    parameter.ref_param = true;
                },
                "const&&" => {
                    parameter.const_param = true;
                    parameter.move_param = true;
                },
                "enum" => (),
                "class" => (),
                "struct" => (),
                "&" => parameter.ref_param = true,
                "*" => parameter.ptr_param = true,
                "&&" => parameter.move_param = true,
                other => {
                    if let Some(index) = other.find("&&"){
                        parameter.type_str = other[0..index].to_string();
                        parameter.move_param = true;
                    }
                    else if let Some(index) = other.find("&"){
                        parameter.type_str = other[0..index].to_string();
                        parameter.ref_param = true;
                    }
                    else if let Some(index) = other.find("*"){
                        parameter.type_str = other[0..index].to_string();
                        parameter.ptr_param = true;
                    }
                    else{
                        if parameter.name.is_empty(){
                            parameter.name = other.to_string();
                        }
                        else{
                            parameter.type_str = other.to_string();
                        }
                    }
                }
            }
        }
        //parameter with no name
        if parameter.type_str.is_empty(){
            parameter.type_str = std::mem::take(&mut parameter.name);
        }
        if parameter.name.is_empty(){
            parameter.name = format!("arg{}", arg_index);
        }
        arg_index += 1;

        assert!(
            !parameter.type_str.is_empty() && 
            !parameter.name.is_empty(), 
            "unknown parameter style {:?}\r\n {:?}", param, parameters
        );
        let (type_str, vt) = parse_c_type(&parameter.type_str);
        parameter.r_type = type_str;
        parameter.param_type = vt as i32;
        list.push(parameter);
    }
    list
}
fn parse_function(api: &mut CppApi, fn_impl: &Vec<String>){
    //
    //println!("fn_impl {:?}", fn_impl);
    let mut iter = fn_impl.iter().rev();
    api.name = iter.next().cloned().expect(&format!("parse function fail {:?}", fn_impl));
    if api.class_name == api.name{
        api.is_construstor = true;
        api.rc_type = api.class_name.clone();
        return;
    }
    if api.name.contains("~"){
        api.is_destructor = true;
        return;
    }
    while let Some(flag) = iter.next() {
        match flag.to_lowercase().as_str(){
            "static" => api.is_static = true,
            "const" => api.is_const = true,
            "const*" => api.is_const = true,
            "virtual" => api.is_virtual = true,
            "friend" => (), //private?
            "stdcall" |
            "constexpr" | "extern" | "struct" | "class" => (),
            _ => {
                if !api.rc_type.is_empty(){
                    panic!("unexpected fn impl flag {}:{} {:?}", flag, api.class_name, fn_impl)
                }
                else{
                    api.rc_type = flag.clone();
                    if let Some(index) = api.rc_type.find("*"){
                        api.rc_type = api.rc_type[0..index].to_owned();
                        api.ptr_ret = true;
                    }
                }
            }
        }
    }
    assert!(!api.rc_type.is_empty() && !api.name.is_empty(), "unexpected api {} pattern {:?}", api.class_name, fn_impl);
    let (type_str, vt) = parse_c_type(&api.rc_type);
    api.r_type = type_str;
    api.return_type = vt as i32;
}
#[inline]
pub fn is_false(val: &bool) -> bool{
    !*val
}