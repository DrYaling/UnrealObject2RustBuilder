#![allow(non_snake_case)]
use std::{path::Path, sync::{Mutex, Arc}, fmt::{Debug}};
use serde::Deserialize;

use super::{unreal_engine::{Engine, UnrealClass}, config::{CppEnum, CppApi, Parameter, CppProperty, CppEnumConstant}};
pub type Node = clang_ast::Node<Clang>;

#[derive(Deserialize)]
pub struct Clang {
    pub kind: clang_ast::Kind,  // or clang_ast::Kind
    pub loc: Option<clang_ast::SourceLocation>,
    pub range: Option<clang_ast::SourceRange>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub isInvalid: bool,
    #[serde(default)]
    pub storageClass: Option<String>,
    pub r#type: Option<QualType>,
    #[serde(default)]
    pub access: String,
    #[serde(default)]
    pub tagUsed: String,
    #[serde(default)]
    pub value: serde_json::Value,
    pub scopedEnumTag: Option<String>,
    #[serde(default)]
    pub explicitlyDeleted: bool,

}
impl Debug for Clang{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clang")
        .field("kind", &self.kind)
        .field("loc", &self.loc).field("range", &self.range)
        .field("name", &self.name).field("isInvalid", &self.isInvalid)
        .field("storageClass", &self.storageClass)
        // .field("r#type", &self.r#type)
        .field("access", &self.access).field("tagUsed", &self.tagUsed)
        .field("value", &self.value).field("scopedEnumTag", &self.scopedEnumTag)
        .field("explicitlyDeleted", &self.explicitlyDeleted).finish()
    }
}
#[derive(Deserialize)]
pub struct QualType {
    qualType: Option<String>
}
pub fn run(engine: &mut Engine) -> anyhow::Result<()>{
    //https://www.cnblogs.com/kuliuheng/p/10769192.html
    //clang -Xclang -ast-dump -fsyntax-only -Iinclude -x c++ test.h > out.txt

    //clang -Xclang -dump-tokens -Iinclude test.h > out2.txt
    //clang -fsyntax-only -Xclang -ast-dump -x c++ -Iinclude test.h > out3.txt
    //https://blog.csdn.net/hatter110/article/details/107282596

    //clang -Xclang -ast-dump=json -fsyntax-only -x c++ engine_code/Actor.h > out.txt
    let current_dir = std::env::current_dir()?;
    let files = crate::read_files("engine_code\\", "*.h")?;
    let target_dir = current_dir.join("unreal_ast").display().to_string().replace("/", "\\");
    std::fs::create_dir(&target_dir).ok();
    let thread_count = 
    // 1;
    num_cpus::get();
    let step = files.len() / thread_count;
    let mut threads = vec![];
    let mut engines = vec![];
    for index in 0..files.len() / thread_count {
        let mut local_files = Vec::with_capacity(step);
        for i in index*step..(index+1)*step.min(files.len()) {
            if i < files.len(){
                local_files.push(files[i].clone());
            }
        }  
        let shared_engine = Arc::new(Mutex::new(Engine::default()));
        engines.push(shared_engine.clone());
        let target_dir = target_dir.clone();
        let current_dir = current_dir.clone();
        threads.push(std::thread::spawn(move ||{
            let block = || -> anyhow::Result<()>{
                for file in &local_files {
                    // if !file.ends_with("GameplayStatics.h"){
                    //     continue;
                    // }
                    // let file_name = Path::new(&file).file_stem().unwrap().to_str().unwrap();
                    let file_path = current_dir.join(&file).display().to_string().replace("/", "\\");
                    let target_file_path = file.replace("\\", "/").replace("engine_code/", "");
                    let target_path = format!("{}\\{}.json", &target_dir, target_file_path);
                    let out_file =if let Ok(c)= std::fs::read_to_string(&target_path){
                        // println!("parse ast file {target_path}");
                        c
                    }
                    else{
                        let cmd = format!(
                            "clang -Xclang -ast-dump=json -fsyntax-only -x c++ -std=c++17 {}",
                            file_path//, target_path
                        );
                        // println!("thread {index} cmd {}", cmd);
                        let output = std::process::Command::new("powershell")
                        .arg("/c")
                        .arg(&cmd)
                        .output()?;
                        let out_file = unsafe{ String::from_utf8_unchecked(output.stdout)};
                        if !output.status.success() && out_file.is_empty(){
                            println!("file {} cmd {} result {:?}", file_path, output.status, unsafe{ String::from_utf8_unchecked(output.stderr)});
                            continue;
                        }
                        out_file
                    };                    
                    let ast: Node = serde_json::from_str(&out_file)?;
                    std::fs::create_dir_all(Path::new(&target_path).parent().unwrap()).ok();
                    std::fs::write(target_path, out_file)?;
                    // println!("cmd ast {}", ast.id.to_string());
                    if let Err(e) = parse_file(&ast, &file_path, &mut shared_engine.lock().unwrap()){
                        println!("parse file {file_path} ast fail {:?}", e);
                    } 
                    // break;
                }
                Ok(())
            };
            if let Err(e) = block(){
                println!("fail to parse fiels {:?}", e);
            }
        }));
    }
    for t in threads {
        t.join().ok();
    }
    for e in engines{
        let mut et = e.lock().unwrap();
        engine.classes.append(&mut et.classes);
        engine.enums.append(&mut et.enums);
        engine.static_apis.append(&mut et.static_apis);
    }
    // let mut command = std::process::Command::new("powershell");
    // command.arg("chcp 65001\r\n").output().ok();
    Ok(())
}
#[derive(Debug, Default)]
struct ParseState{
    is_pub: bool,
    is_struct: bool,
    is_class: bool,  
    content: String,
    file_path: String,
}
impl ParseState{
    fn clear(&mut self){
        self.is_class = false;
        self.is_struct = false;
        self.is_pub = false;   
    }
}
#[inline]
fn get_file_path(fp: &str) -> String{
    let mut cur = std::env::current_dir().unwrap().display().to_string().replace("\\", "/");
    if !cur.ends_with("/"){
        cur.push_str("/");
    }
    fp.replace("\\", "/").replace(&cur, "")
    .replace("engine_code/", "")
}
///parse file with ast
fn parse_file(ast: &Node, file_path: &str, engine: &mut Engine) -> anyhow::Result<()>{
    let mut state = ParseState::default();
    state.content = std::fs::read_to_string(file_path)?;
    if state.content.is_empty(){
        return Ok(());
    }
    // let file_name = std::path::Path::new(file_path).file_name().unwrap().to_str().unwrap().to_string();
    // if file_path.ends_with("GameplayStatics.h"){
    //     println!("pause");
    // }
    state.file_path = get_file_path(file_path);
    parse_node(ast, engine, &mut state)?;
    Ok(())
}
///parse recursively from node root
fn parse_node(ast: &Node, engine: &mut Engine, state: &mut ParseState) -> anyhow::Result<()>{
    state.clear();
    match ast.kind.kind {
        clang_ast::Kind::CXXRecordDecl => {
            let kind = &ast.kind;
            match kind.name.as_str(){
                "_GUID" | "type_info"  => (),
                _ => {                    
                    match kind.tagUsed.as_str() {
                        "class" => {
                            state.is_class = true;
                            if let Some(mut clas) = parse_class(&ast, state)?{
                                clas.path = state.file_path.clone();
                                clas.public_apis.sort_by_key(|api| api.name.clone());
                                clas.public_apis.dedup_by(|a, b| {
                                    a.name == b.name && a.parameters.len() == b.parameters.len()
                                });
                                engine.classes.push(clas);
                            }
                        },
                        "struct" => {
                            state.is_struct = true;
                            if let Some(mut clas) = parse_class(&ast, state)?{
                                clas.path = state.file_path.clone();
                                clas.public_apis.sort_by_key(|api| api.name.clone());
                                clas.public_apis.dedup_by(|a, b| {
                                    a.name == b.name && a.parameters.len() == b.parameters.len()
                                });
                                engine.classes.push(clas);
                            }
                        }
                        _ => (),
                    }
                }
            }
        },
        clang_ast::Kind::EnumDecl => {
            if let Some(eu) = parse_enum(ast, state)?{
                engine.enums.push(eu);
            }
        },
        //global const
        clang_ast::Kind::FunctionDecl => {
            if let Some(api) = parse_api(ast, state)?{
                engine.static_apis.push(api);
                engine.static_apis.sort_by_key(|api| api.name.clone());
                engine.static_apis.dedup_by(|a, b| {
                    a.name == b.name && a.parameters.len() == b.parameters.len()
                });
            }
        },
        _ => {                
            for inner in &ast.inner {        
                parse_node(inner, engine, state)?;
            }
        },
    }
    Ok(())
}
///parse enum
/// int enum only
fn parse_enum(node: &Node, _state: &mut ParseState) -> anyhow::Result<Option<CppEnum>>{
    let mut cenum= CppEnum{
        name: node.kind.name.clone(),
        enum_class: node.kind.scopedEnumTag.as_ref().map(|tag| tag == "class").unwrap_or_default(),
        ..Default::default()
    };
    let mut value: i32 = 0;
    for node in &node.inner {
        if let clang_ast::Kind::EnumConstantDecl= node.kind.kind{
            //int value not ref
            let mut valid = false;
            if let Some(QualType { qualType: Some(qt) }) = &node.kind.r#type {
                if qt == &cenum.name{
                    valid = true;
                }
            }
            if !valid{
                //reference enum not supported yet
                return Ok(None);
            }
            let mut ec = CppEnumConstant{
                name: node.kind.name.clone(),
                value: value,
            };
            if let Some(expr) =node.inner.iter().next(){
                if let clang_ast::Kind::ConstantExpr = expr.kind.kind{
                    if let serde_json::Value::String(vs) = &expr.kind.value{
                        if let Ok(v) = vs.parse::<i32>(){
                            ec.value = v;
                            value = v;
                        }
                        else{
                            //unrecongenized value
                            return Ok(None);
                        }
                    }
                }
                else{
                    //unrecongenized value
                    return Ok(None);
                }
            }
            cenum.constants.push(ec);
            value += 1;
        }
    }
    Ok(Some(cenum))
}
///parse class
fn parse_class(node: &Node, state: &mut ParseState) -> anyhow::Result<Option<UnrealClass>>{
    macro_rules! none_pub {
        ($is_pub: expr) => {
            if !$is_pub{ continue;}
        };
    }
    //empty class decl
    if node.inner.len() == 0{
        return Ok(None);
    }
    let mut class = UnrealClass{
        is_struct: state.is_struct,
        name: node.kind.name.clone(),
        ..Default::default()
    };
    state.is_pub = state.is_struct;
    for node in &node.inner {
        let kind = &node.kind; 
        match kind.kind {
            clang_ast::Kind::AccessSpecDecl => {
                state.is_pub = kind.access == "public";
            },
            clang_ast::Kind::CXXConstructorDecl |
            clang_ast::Kind::CXXMethodDecl => {
                none_pub!(state.is_pub);
                if let Some(mut api) = parse_api(node, state)?{
                    api.class_name = class.name.clone();
                    if let Some(exist)= class.public_apis.iter().find(|a| a.name == api.name){
                        if exist.parameters.len() != api.parameters.len(){
                            class.public_apis.push(api);
                        }
                    }
                    else{
                        class.public_apis.push(api);
                    }
                }
            },
            clang_ast::Kind::FieldDecl => {
                let field = parse_field(kind, state)?;
                if state.is_pub{
                    class.properties.push(field);
                }
                else{
                    class.none_public_properties.push(field);
                }
            },
            clang_ast::Kind::CXXRecordDecl => {
                if kind.tagUsed == "union"{
                    parse_union(node, &mut class, state)?;
                }
            }
            _ => (),
        }
    }
    Ok(Some(class))
}
//parse union to property at this moment
fn parse_union(node: &Node, class: &mut UnrealClass, state: &mut ParseState) -> anyhow::Result<()>{
    //only struct is supported
    if !class.is_struct{
        return Ok(());
    }
    //parse first stuct
    for child in &node.inner {
        if child.kind.kind == clang_ast::Kind::CXXRecordDecl && child.kind.tagUsed == "struct"{
            if let Some(mut structure) = parse_class(child, state)?{
                class.properties.append(&mut structure.properties);
                class.none_public_properties.append(&mut structure.none_public_properties);
                break;
            }
        }
    }
    Ok(())
}
///parse api
fn parse_api(node: &Node, state: &ParseState) -> anyhow::Result<Option<CppApi>>{
    let kind = &node.kind;
    if kind.explicitlyDeleted || kind.name.starts_with("operator"){
        return Ok(None);
    }
    let mut api = CppApi{
        name: node.kind.name.clone(),
        is_construstor: kind.kind == clang_ast::Kind::CXXConstructorDecl,
        ..Default::default()
    };
    // if api.name == "DeprojectScreenToWorld"{
    //     println!("pause");
    // }
    parse_parm_decl(&node.inner, &mut api, state)?;
    if let Some(sc)= &kind.storageClass{
        if sc == "static"{
            api.is_static = true;
        }                
    }
    //result type
    if let (Some(Some(start)), Some(Some(end))) = (
        kind.range.as_ref().map(|range| &range.begin.expansion_loc),
        kind.loc.as_ref().map(|r| &r.spelling_loc)
    ){
        api.rc_type = state.content[start.offset..end.offset].trim().to_string();
    }
    else{
        //api invalid
        println!("invalid api return type {}", api.name);
        return Ok(None);
    }
    if api.rc_type.contains("static ") || api.rc_type.contains("static\t"){
        api.rc_type = api.rc_type.replace("static", "").trim().to_string();
        api.is_static = true;
    }
    if api.rc_type.contains("class ") || api.rc_type.contains("class\t"){
        api.rc_type = api.rc_type.replace("class", "").trim().to_string();
    }
    if api.rc_type.contains("struct ") || api.rc_type.contains("struct\t"){
        api.rc_type = api.rc_type.replace("struct", "").trim().to_string();
    }
    //const api
    if let Some(QualType { qualType: Some(qt) }) = &kind.r#type {
        if qt.ends_with("const"){
            api.is_const = true;
        }
    }
    if let Some(index) = api.rc_type.find("virtual"){
        if index > 0{
            api.rc_type = format!("{}{}", &api.rc_type[0..index], &api.rc_type[index + 7..]).trim().to_string();
        }
        else{
            api.rc_type = api.rc_type[index + 7..].trim().to_string();
        }
        api.is_virtual = true;
    }
    //const result
    if let Some(index) = api.rc_type.find("const"){
        api.rc_type = api.rc_type[index + 5..].trim().to_string();
        api.const_ret = true;
    }
    if let Some(index) = api.rc_type.find("&"){
        api.rc_type = api.rc_type[..index].trim().to_string();
        api.ref_ret = true;
    }
    else if let Some(index) = api.rc_type.find("*"){
        api.rc_type = api.rc_type[..index].trim().to_string();
        api.ptr_ret = true;
    }
    api.is_generic = api.parameters.iter().find(|p| p.is_generic).is_some();
    if !api.is_generic{
        if let (Some(_), Some(_)) = (api.rc_type.find("<"), api.rc_type.find(">")){
            api.is_generic = true;
        }
    }
    let (r_s, vt) = super::parse_c_type(&api.rc_type);
    api.r_type = r_s;
    api.return_type = vt as i32;
    if api.invalid{
        Ok(None)
    }
    else{
        Ok(Some(api))
    }
}
fn is_bit_value(state: &ParseState, mut offset: usize) -> bool{
    offset += 1;
    while offset < state.content.len() {
        let c = &state.content[offset..offset + 1];
        if c == ";" { return false;}
        else if c == ":" {return true;}
        offset += 1;
    }
    false
}
///parse field
fn parse_field(kind: &Clang, state: &ParseState) -> anyhow::Result<CppProperty>{    
    let mut field = CppProperty{
        name: kind.name.clone(),                    
        ..Default::default()
    };
    if kind.isInvalid{
        if let Some((Some(start), Some(end))) = kind.range.as_ref().map(|range| (&range.begin.expansion_loc, &range.end.expansion_loc)){
            field.type_str = state.content[start.offset..end.offset].trim().to_string();
        }
    }
    else{
        field.type_str = kind.r#type.as_ref().map(|qual| qual.qualType.clone().unwrap_or_default()).unwrap_or_default();        
    }
    if let Some(index) = field.type_str.find("*"){
        field.type_str = field.type_str[..index].trim().to_string();
        field.is_ptr = true;
    }
    if let (Some(_), Some(_)) = (field.type_str.find("<"), field.type_str.find(">")){
        field.is_generic = true;
    }
    field.bit_value = is_bit_value(
        state, 
        kind.loc.as_ref()
        .map(|loc| loc.expansion_loc.as_ref().map(|loc| loc.offset))
        .unwrap_or_default().unwrap_or_default()
    );
    let (r_s, vt) = super::parse_c_type(&field.type_str);
    field.r_type = r_s;
    field.value_type = vt as i32;
    if let Some(sc)= &kind.storageClass{
        if sc == "static"{
            field.is_static = true;
        }                
    }
    Ok(field)
}
fn parse_parm_decl(inner: &Vec<Node>, api: &mut CppApi, state: &ParseState) -> anyhow::Result<()>{
    let mut params: Vec<Parameter> = Vec::with_capacity(inner.len());
    let mut arg_count: usize = 0;
    for node in inner {
        let kind = &node.kind;
        if let clang_ast::Kind::ParmVarDecl = kind.kind{
            let mut param = Parameter{
                name: kind.name.clone(),                
                ..Default::default()
            };
            if let Some((Some(start), Some(end))) = kind.range
            .as_ref()
            .map(|range| 
                (
                    range.begin.expansion_loc.clone().or(range.begin.spelling_loc.clone()), 
                    range.end.expansion_loc.clone().or(
                        range.end.spelling_loc.clone().or(kind.loc.as_ref().map(|s| s.spelling_loc.clone().or(s.expansion_loc.clone())).unwrap_or_default()
                    ))
                )
            ){
                if start.offset >= end.offset{
                    return Err(anyhow!("api {:?} unrecongnized with offset incorrect {:?}", api, kind));
                }
                param.type_str = state.content[start.offset..end.offset].trim().to_string();
                //remove =
                if param.name.contains("="){
                    param.name = param.name[0..param.name.find("=").unwrap()].trim().to_string();
                }
                //remove =
                if let Some(index) = param.type_str.find("="){
                    param.type_str = param.type_str[0..index].trim().to_string();
                }
                //remove const(at most 2?)
                if param.type_str.starts_with("const"){
                    if let Some(index) = param.type_str.find("const"){
                        let next = index + 5;
                        if next < param.type_str.len(){
                            if is_space(&param.type_str[next..=next]){                            
                                param.type_str = param.type_str[next..].trim().to_string();
                                param.const_param = true;
                            }
                        }
                    }
                }
                if param.type_str.ends_with("const"){
                    if let Some(index) = param.type_str.rfind("const"){
                        if index >= 1{
                            let prev = index -1;
                            if is_space(&param.type_str[prev..=prev]){                            
                                param.type_str = param.type_str[0..prev].trim().to_string();
                                param.const_param = true;
                            }
                        }
                    }
                }
                if param.type_str.ends_with("const*"){
                    if let Some(index) = param.type_str.rfind("const*"){                        
                        param.type_str = param.type_str[0..index].trim().to_string();
                        param.const_param = true;
                        param.ptr_param = true;
                    }
                }
                if param.type_str.ends_with("const&"){
                    if let Some(index) = param.type_str.rfind("const&"){                    
                        param.type_str = param.type_str[0..index].trim().to_string();
                        param.const_param = true;
                        param.ref_param = true;
                    }
                }
                if param.type_str.contains("class ") || param.type_str.contains("class\t"){
                    param.type_str = param.type_str.replace("class", "").trim().to_string();
                }
                if param.type_str.contains("struct ") || param.type_str.contains("struct\t"){
                    param.type_str = param.type_str.replace("struct", "").trim().to_string();
                }
                if let Some(index) = param.type_str.find("&&"){
                    param.type_str = param.type_str[..index].trim().to_string();
                    param.move_param = true;
                }
                else if let Some(index) = param.type_str.find("&"){
                    param.type_str = param.type_str[..index].trim().to_string();
                    param.ref_param = true;
                }
                else if let Some(index) = param.type_str.find("*"){
                    param.type_str = param.type_str[..index].trim().to_string();
                    param.ptr_param = true;
                }
                if let (Some(_), Some(_)) = (param.type_str.find("<"), param.type_str.find(">")){
                    param.is_generic = true;
                }
                //some api can not split type and name
                let mut types = param.type_str.split(|c| c == ' ' || c == '\t').map(|s| s.to_string()).collect::<Vec<_>>();
                if types.len() > 1{
                    param.name = types.remove(types.len() - 1).trim().to_string();
                    param.type_str = types.join(" ").trim().to_string();
                }
                let (r_s, vt) = super::parse_c_type(&param.type_str);
                param.r_type = r_s;
                param.param_type = vt as i32;
                if param.name.is_empty(){
                    param.name = format!("arg{}", arg_count);
                }
                params.push(param);
            }
            else{
                println!("fail to parse parameter {} param {}", api.name, kind.name);
                api.invalid = true;
                return Ok(());
            }
            arg_count += 1;
        }
    }
    api.parameters = params;
    Ok(())
}
#[inline]
fn is_space(index: &str) -> bool {
    index == " " || index == "\t"
}