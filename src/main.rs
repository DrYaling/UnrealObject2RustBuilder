#[allow(unused)]
#[macro_use] 
extern crate serde_json;
mod api;
mod object;
mod property;
mod parser;
mod const_parser;
mod enum_parser;
mod object_parser;
use std::collections::BTreeMap;

use object::UnrealObject;
use once_cell::sync::Lazy;

pub static C_PRIMARY_TYPE: Lazy<BTreeMap<String, String>> = Lazy::new(||{
    let map = 
    vec![
        ("i32".into(),"int32_t".into()), 
        ("u32".into(), "uint32_t".into()), 
        ("u64".into(), "uint64_t".into()),
        ("i64".into(), "int64_t".into()), 
        ("usize".into(), "size_t".into()), 
        ("f32".into(), "float".into()),
        ("f64".into(), "double".into()), 
        ("i16".into(),"int16_t".into()), 
        ("u16".into(),"uint16_t".into()), 
        ("u8".into(), "unsigned char".into()), 
        ("i8".into(), "char".into()),
        ("bool".into(), "bool".into())
    ].into_iter().collect();
    map
});
fn read_files(path: &str, pattern: &str) -> anyhow::Result<Vec<String>> {
    let files = std::fs::read_dir(path)?;    //读出目录
    let mut output = Vec::new();
    for path in files {
        let entry = path?;
        if entry.file_type()?.is_dir(){
            output.append(&mut read_files(entry.path().as_path().to_str().unwrap(), pattern)?);
        }
        else{
            if pattern.len() > 0{
                match entry.path().extension(){
                    Some(ext) => {
                        if !pattern.contains(ext.to_str().unwrap()){
                            continue;
                        }
                    },
                    None => continue,
                }
            }
            let path = entry.path().as_path().to_str().unwrap().replace("\\", "/").replace("\\\\", "/");
            output.push(path);
        }
    }
    Ok(output)
}
fn load_binders(path: &str) -> anyhow::Result<Vec<UnrealObject>>{
    let files = read_files(path, "json")?;
    let mut output = Vec::new();
    for file in files {
        let json = std::fs::read_to_string(file)?;
        output.push(serde_json::from_str(&json)?);
    }
    Ok(output)
}
fn main() -> anyhow::Result<()> {
    let objects = load_binders("Binders")?;

    // println!("Hello, world! {:?}", objects);
    parser::parse(objects)?;
    Ok(())
}
