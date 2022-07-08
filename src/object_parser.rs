use std::collections::HashMap;

use crate::{object::UnrealObject, parser::CppBinderApi};

///parse enum value
pub fn parse_obj(obj: &UnrealObject, rust_binders: &mut HashMap<String, Vec<String>>, _cpp_binders: &mut HashMap<String, Vec<CppBinderApi>>) -> anyhow::Result<()>{
    let obj_name = if obj.alias.is_empty(){
        obj.unreal_type.as_str()
    }
    else{
        obj.alias.as_str()
    };
    let mut lines = vec![
        format!("///rust api for unreal type {}", obj.unreal_type),
        format!("pub struct {}{{", obj_name),        
    ];

    if obj.opaque_type{

        for api in &obj.apis {
        
        }
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
        //
        if obj.apis.len() > 0{
            panic!("struct api unsupported yet")
        }
    }
    lines.push("}".into());
    rust_binders.insert(obj_name.to_lowercase(), lines);
    Ok(())
}