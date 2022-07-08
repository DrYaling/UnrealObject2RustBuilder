use std::collections::HashMap;

use crate::{object::UnrealObject, parser::CppBinderApi};

///parse enum value
pub fn parse_enum(_obj: &UnrealObject, _rust_binders: &mut HashMap<String, Vec<String>>, _cpp_binders: &mut HashMap<String, Vec<CppBinderApi>>) -> anyhow::Result<()>{
    panic!("enum exporter is not supported");
    // Ok(())
}