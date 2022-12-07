#[allow(unused)]
#[macro_use] 
extern crate serde_json;
// include!("../Binders/rs/binders.rs");
// mod api;
// mod object;
// mod property;
// mod parser;
// mod const_parser;
// mod enum_parser;
// mod binders;
mod engine_parser;
use std::collections::BTreeMap;

use engine_parser::ValueType;
// use object::UnrealObject;
use once_cell::sync::Lazy;
fn load_primaries() -> Vec<(String, String)>{
    vec![
        ("i32".into(),"int32".into()), 
        ("i64".into(), "int64".into()), 
        ("u32".into(), "uint32".into()), 
        ("u64".into(), "uint64".into()),
        ("i64".into(), "int64".into()),
        ("usize".into(), "size_t".into()), 
        ("f32".into(), "float".into()),
        ("f64".into(), "double".into()), 
        ("i16".into(),"int16".into()), 
        ("u16".into(),"uint16".into()), 
        ("u8".into(), "uint8".into()), 
        ("i8".into(), "int8".into()),
        ("bool".into(), "bool".into()),
        ("()".into(), "void".into())
    ]
}
pub static RUST_TO_C_TYPES: Lazy<BTreeMap<String, (String, engine_parser::ValueType)>> = Lazy::new(||{
    let map = 
    load_primaries()
    .into_iter()
    .enumerate()
    .map(|(index, (a, b))|{
        (a, (b, ValueType::from(index as i32)))
    })
    .collect();
    map
});

static CPP_TO_RUST_TYPES: Lazy<BTreeMap<String, (String, engine_parser::ValueType)>> = Lazy::new(||{
    let map = 
    load_primaries()
    .into_iter()
    .map(|(a,b)| (b, a))    
    .enumerate()
    .map(|(index, (a, b))|{
        (a, (b, ValueType::from(index as i32)))
    })
    .collect();
    map
});
pub fn get_c2r_types(key: &str) -> Option<(String, ValueType)>{
    CPP_TO_RUST_TYPES.get(key).cloned()
    .or(CPP_TO_RUST_TYPES.get(&format!("{key}_t")).cloned())
}
pub fn is_rs_primary(type_str: &str) -> bool{    
    RUST_TO_C_TYPES.get(type_str).is_some()
}
pub fn is_primary(type_str: &str) -> bool{    
    CPP_TO_RUST_TYPES.get(type_str)
    .or(CPP_TO_RUST_TYPES.get(&format!("{type_str}_t"))).is_some()
}
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
// fn load_binders(path: &str) -> anyhow::Result<Vec<UnrealObject>>{
//     let files = read_files(path, "json")?;
//     let mut output = Vec::new();
//     for file in files {
//         let json = std::fs::read_to_string(file)?;
//         output.push(serde_json::from_str(&json)?);
//     }
//     Ok(output)
// }
fn main() -> anyhow::Result<()> {
    // crate::ast::run()?;
    // return Ok(());
    let ins = std::time::Instant::now();
    engine_parser::class_parser::parse(
        serde_json::from_reader(
            std::fs::File::open("configs/CustomSettings.json")?
        )?
    ).map_err(|e| {println!("{:?}", e); e})?;
    println!("run finish cost {}", ins.elapsed().as_secs_f64());
    // let objects = load_binders("binders")?;

    // // println!("Hello, world! {:?}", objects);
    // parser::parse(objects)?;
    // let mut str_buf: String = String::new();
    // std::io::stdin().read_line(&mut str_buf)?;
    Ok(())
}
