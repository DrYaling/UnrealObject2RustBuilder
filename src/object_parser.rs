use std::{collections::{HashMap, BTreeMap}, sync::Mutex};

use once_cell::sync::Lazy;

use crate::{object::UnrealObject, parser::CppBinderApi, RUST_TO_C_TYPES};
static ALIASES: Lazy<Mutex<BTreeMap<String, String>>> = Lazy::new(|| Mutex::new(BTreeMap::new())); 
fn add_alias(alias: &str, obj_type: &str){
    ALIASES.lock().unwrap().insert(alias.to_string(), obj_type.to_string());
}
fn get_origin_name(name: &str) -> String {
    ALIASES.lock().unwrap().get(name).cloned().unwrap_or(name.to_string())
}
static mut OPAQUE_OBJECTS: Vec<String> = Vec::new(); 
fn add_opaque(tp: &str){
    unsafe{
        OPAQUE_OBJECTS.push(tp.to_string());
    }
}
fn is_opaque_type(tp: &str)-> bool {
    if unsafe{ OPAQUE_OBJECTS.contains(&tp.replace("&", "")) }{
        true
    }
    else if unsafe{ OPAQUE_OBJECTS.contains(&tp.replace("&mut", "")) }{
        true
    }
    else{
        false
    }
}
fn get_obj_type(tp: String) -> String{
    if unsafe{ OPAQUE_OBJECTS.contains(&tp.replace("&", "")) }{
        "*const std::ffi::c_void".to_string()
    }
    else if unsafe{ OPAQUE_OBJECTS.contains(&tp.replace("&mut", "")) }{
        "*mut std::ffi::c_void".to_string()
    }
    else{
        tp
    }
}
fn get_obj_name(name: String, obj_type:&str) -> String{
    if unsafe{ OPAQUE_OBJECTS.contains(&obj_type.replace("&", "")) }{
        format!("{}.get_ptr()", name)
    }
    else if unsafe{ OPAQUE_OBJECTS.contains(&obj_type.replace("&mut", "")) }{
        format!("{}.get_ptr_mut()", name)
    }
    else{
        name
    }
}
///parse enum value
pub fn parse_obj(obj: &UnrealObject, rust_binders: &mut HashMap<String, Vec<String>>, cpp_binders: &mut HashMap<String, Vec<CppBinderApi>>) -> anyhow::Result<()>{
    let obj_name = if obj.alias.is_empty(){
        obj.unreal_type.as_str()
    }
    else{
        add_alias(&obj.alias, &obj.unreal_type);
        obj.alias.as_str()
    };
    let mut cpp_bindings = Vec::new();
    let mut lines = vec![
        format!("///rust api for unreal type {}", obj.unreal_type),
        format!("pub struct {}{{", obj_name),        
    ];

    if obj.opaque_type{
        lines.push("\tptr: *mut std::ffi::c_void".into());
        add_opaque(obj_name);
    }
    else{
        if obj.r#type == "class"{
            panic!("class of not opaque_type not supported yet")
        }
        else {
            for property in &obj.properties {
                lines.push(format!("\tpub {}: {},", property.name, property.r#type));
            }
        }
    }
    lines.push("}".into());
    let mut impl_funs = vec![];
    impl_funs.push(format!("impl {}{{", obj_name));
    if obj.opaque_type{
        impl_funs.push(format!("\t#[inline] pub fn get_ptr(&self) -> *const std::ffi::c_void {{ self.ptr as *const _ }}"));
        impl_funs.push(format!("\t#[inline] pub fn get_ptr_mut(&self) -> *mut std::ffi::c_void {{ self.ptr }}"));
    }
    for api in &obj.apis {
        //pub type UELogCallback = unsafe extern "C" fn(data: *const c_char, log_level: i32);
        let callback_name = format!("{}{}", obj_name, api.name);
        //(type, name)
        let mut callback_parameters: Vec<(String, String)> = vec![
            //("*mut std::ffi::c_void".into(), "this".into())
        ];
        let mut c_api = CppBinderApi::default();
        for param in &api.parameters {
            callback_parameters.push((
                param.r#type.clone(), param.name.clone()
            ));
        }
        let result = if api.ret_type.len() > 0{
            format!(" -> {}", api.ret_type)
        }
        else{
            String::default()
        };
        let func_type = format!("{}FPtr", callback_name);
        let cb_params = if callback_parameters.len() > 0{
            format!("this: *mut std::ffi::c_void, {}", 
            callback_parameters.iter().map(|t| format!("{}: {}", t.1, get_obj_type(t.0.clone()))).collect::<Vec<_>>().join(", "))
        }
        else{
            "this: *mut std::ffi::c_void".into()
        };
        let cfp_params = if api.parameters.len() > 0{
            format!("{}* ptr, {}", obj.unreal_type,
            api.parameters.iter().map(|t| {
                match RUST_TO_C_TYPES.get(&t.r#type){
                    Some(ct) => format!("{} {}", ct.0, t.name),
                    None => {
                        let ref_type = t.r#type.contains("&");
                        let alias = t.r#type.replace("&", "");
                        let unreal_type = get_origin_name(&alias);
                        if is_opaque_type(&unreal_type){
                            format!("{}* {}", unreal_type, t.name)
                        }
                        else if ref_type{
                            format!("{}& {}", unreal_type, t.name)
                        }
                        else{
                            format!("{} {}", unreal_type, t.name)
                        }
                    },
                }
            }).collect::<Vec<_>>().join(", "))
        }
        else{
            format!("{}* ptr", obj.unreal_type)
        };
        lines.push(
            format!("pub type {} = unsafe extern \"C\" fn ({}){};", 
            func_type, cb_params, result
        ));
        c_api.fptr= Some(
            format!("typedef {}(*{})({});", api.ret_type, func_type, cfp_params)
        );
        //cpp binder
        {
            let cb_body = if api.ret_type.len() > 0{
                format!("return ptr->{}({});", api.name, api.parameters.iter().map(|param| param.name.clone()).collect::<Vec<_>>().join(", "))
            }
            else{
                format!("ptr->{}({});", api.name, api.parameters.iter().map(|param| param.name.clone()).collect::<Vec<_>>().join(", "))
            };
            c_api.rust_api = format!("Register{}([]({}){{{}}});", callback_name, cfp_params, cb_body);
        }
        lines.push(format!("static mut {}Callback: Option<{}> = None;", callback_name, func_type));
        lines.push("/// not to lua".to_string());
        lines.push("#[no_mangle]".to_string());
        lines.push(format!("unsafe extern \"C\" fn Register{}(fun: {}){{", callback_name, func_type));
        lines.push(format!("\t{}Callback = Some(fun);", callback_name));
        lines.push("}".into());
        let parameters = if callback_parameters.len() > 0{
            format!("&self, {}", callback_parameters.iter().map(|t| format!("{}: {}", t.1, t.0)).collect::<Vec<_>>().join(", "))
        }
        else{
            "&self".to_string()
        };
        impl_funs.push(format!("\t pub unsafe fn {}({}){}{{", api.name, parameters, result));
        let parameter_names = if callback_parameters.len() > 0{
            format!("self.ptr, {}", 
            callback_parameters
            .iter()
            .map(|t| format!("{}", get_obj_name(t.1.clone(), t.0.as_str())))
            .collect::<Vec<_>>().join(", "))
        }
        else{
            "self.ptr".to_string()
        };
        impl_funs.push(format!("\t\t{}Callback.as_ref().expect(\"{}::{} not registered!\")({})",
            callback_name, obj.unreal_type, api.name, parameter_names
        ));
        impl_funs.push("\t}".into());
        cpp_bindings.push(c_api);
    }
    impl_funs.push("}".into());
    lines.append(&mut impl_funs);
    let ns = match &obj.namespace {
        Some(ns) if ns.len() > 0 => {
            ns.as_str()
        },
        _ => {
            ""
        },
    };
    match rust_binders.get_mut(ns) {
        Some(list) => list.append(&mut lines),
        None => {
            rust_binders.insert(ns.into(), lines);
        },
    }
    cpp_binders.insert(obj.unreal_type.clone(), cpp_bindings);
    Ok(())
}