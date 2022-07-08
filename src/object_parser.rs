use std::collections::HashMap;

use crate::{object::UnrealObject, parser::CppBinderApi};

///parse enum value
pub fn parse_obj(_obj: &UnrealObject, _rust_binders: &mut HashMap<String, Vec<String>>, _cpp_binders: &mut HashMap<String, Vec<CppBinderApi>>) -> anyhow::Result<()>{
    Ok(())
}