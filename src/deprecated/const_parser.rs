use std::collections::HashMap;

use crate::{object::UnrealObject, parser::CppBinderApi};
const CONST_DEF: &'static str = "unreal_const";
///parse const value
pub fn parse_const(obj: &UnrealObject, rust_binders: &mut HashMap<String, Vec<String>>, _cpp_binders: &mut HashMap<String, Vec<CppBinderApi>>) -> anyhow::Result<()>{
    for const_value in &obj.properties{
        let ov = const_value.value.clone().unwrap_or("Default::default()".into());
        let value = if const_value.r#type.as_str() == "string"{
            format!("\"{}\"", ov)
        }
        else{
            ov
        };
        let v = 
        format!("const {}: {} = {};", const_value.name.to_uppercase(), const_value.r#type, value);
        match rust_binders.get_mut(CONST_DEF) {
            Some(list) => list.push(v),
            None => {
                rust_binders.insert(CONST_DEF.to_string(), vec![
                    v
                ]);
            },
        }
    }
    Ok(())
}