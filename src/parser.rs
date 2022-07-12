use std::{collections::HashMap, env};

use crate::{object::UnrealObject, api::Parameter};
#[derive(Debug, Clone, Default)]
//TODO
pub struct CppBinderApi{
    ///cpp绑定接口的rust接口
    pub rust_api: String,
    ///rust接口的参数列表
    pub parameters: Vec<Parameter>,
    ///回调函数定义
    pub fptr: Option<String>,
    ///绑定返回值,一般情况是void
    pub result: String,
}

pub fn parse(objects: Vec<UnrealObject>) -> anyhow::Result<()>{
    let mut rust_binders: HashMap<String, Vec<String>> = HashMap::new();
    let mut cpp_binders: HashMap<String, Vec<CppBinderApi>> = HashMap::new();
    for object in objects {
        match object.r#type.as_str() {
            "class" | "struct" => {
                super::object_parser::parse_obj(&object, &mut rust_binders, &mut cpp_binders)?;
            },
            "enum" => {
                super::enum_parser::parse_enum(&object, &mut rust_binders, &mut cpp_binders)?;
            },
            "const" => {
                super::const_parser::parse_const(&object, &mut rust_binders, &mut cpp_binders)?;
            },
            _ => panic!("unknown type {:?}", object.r#type),
        }
    }
    generate_rust(rust_binders)?;
    generate_cpp(cpp_binders)?;
    Ok(())
}
fn generate_rust(rust_binders: HashMap<String, Vec<String>>) -> anyhow::Result<()>{
    let rs_path = env::current_dir()?.join("../UnrealRustApi/src/unreal/").as_path().display().to_string().replace("\\", "/");
    println!("rs target path: {}", rs_path);
    let mut mod_file: Vec<String> = vec![
        "//! this is auto generated for unreal bindings by UnrealObject2RustBuilder".into(),
        "#![allow(non_upper_case_globals)]".into(),
        "#![allow(non_camel_case_types)]".into(),
        "#![allow(non_snake_case)]".into(),
        "#![allow(unused)]".into(),
        "#![allow(dead_code)]".into(),
    ];
    std::fs::create_dir_all(&rs_path).ok();
    let cross_types = env::current_dir()?.join("src/cross_types.rs").as_path().display().to_string();
    mod_file.push(std::fs::read_to_string(&cross_types)?);
    mod_file.push("pub type string = &'static str;".to_string());
    for (namespace, mut content) in rust_binders  {
        if namespace.len() > 0{
            mod_file.push(format!("pub use {}::*;", namespace));
            mod_file.push(format!("///{}  binder impl", namespace));        
            mod_file.push(format!("mod {}{{", namespace));
            mod_file.push(format!("\tuse super::*;")); 
            content.iter_mut().for_each(|c| c.insert(0, '\t'));  
            mod_file.append(&mut content);        
            mod_file.push(format!("}}//end of {}", namespace));
        } 
        else{ 
            mod_file.append(&mut content);        
        }
    }
    std::fs::write(rs_path + "/mod.rs", mod_file.join("\r\n"))?;
    Ok(())
}
fn generate_cpp(cpp_binders: HashMap<String, Vec<CppBinderApi>>) -> anyhow::Result<()>{
    let cpp_path = env::current_dir()?.join("../UnrealRustInterface/Plugins/unreal_rs/Source/unreal_rs/Public").as_path().display().to_string().replace("\\", "/");
    println!("cpp target path: {}", cpp_path);
    let mut cpp_file: Vec<String> = vec![
        "//auto generated binding file for UnrealRustApi by UnrealObject2RustBuilder".into(),
        "#include \"CoreMinimal.h\"".into(),
        "#include \"RustBinders.h\"".into(),
        "#include \"core_api.h\"".into(),
    ];
    let mut header: Vec<String> = vec![
        "//auto generated binding file for UnrealRustApi by UnrealObject2RustBuilder".into(),
        "#pragma once".to_string(),
        "void InitBindings();".into()
    ];
    
    cpp_file.push("void InitBindings(){".into());
    for (binder_type, content) in cpp_binders  {
        cpp_file.push(format!("\t//binder {} begin", &binder_type));
        for api in content {
            cpp_file.push(format!("\t{}", api.rust_api));
            if let Some(fptr) = api.fptr{
                header.push(fptr);
            }
        }
        cpp_file.push(format!("\t//binder {} end", &binder_type));
    }
    cpp_file.push("}".into());

    std::fs::write(cpp_path.clone() + "/RustBinders.cpp", cpp_file.join("\r\n"))?;
    std::fs::write(cpp_path + "/RustBinders.h", header.join("\r\n"))?;
    Ok(())
}