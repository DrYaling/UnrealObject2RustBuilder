#![allow(non_snake_case)]
use super::{unreal_engine::{Engine, UnrealClass}, config::{CppEnum, CppApi, Parameter, CppProperty}};

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
    clang_sys::load().unwrap();
    let libclang = clang::Clang::new().unwrap();
    for file in &files {
        if !file.ends_with("/Actor.h"){
            continue;
        }
        let file_path = current_dir.join(&file).display().to_string().replace("/", "\\");
        let indexer = clang::Index::new(&libclang, false, false);
        let mut parser = indexer.parser(&file);
        //-Xclang -ast-dump=json -fsyntax-only -x c++
        parser.arguments(&vec![
            "-Xclang",
            "-ast-dump=json",
            "-x",
            "c++",
        ])
        .cache_completion_results(true)
        .include_attributed_types(true)
        .single_file_parse(true);
        match parser.parse() {
            Ok(trans_unit) => {
                let entity = trans_unit.get_entity();
                parse_file(&entity, &file_path, engine)?; 
            },
            Err(error) => {
                println!("parse file {file_path} error {:?}", error);
            },
        };
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
fn parse_file(ast: &clang::Entity, file_path: &str, engine: &mut Engine) -> anyhow::Result<()>{
    let mut state = ParseState::default();
    state.content = std::fs::read_to_string(file_path)?;
    if state.content.is_empty(){
        return Ok(());
    }
    // let file_name = std::path::Path::new(file_path).file_name().unwrap().to_str().unwrap().to_string();
    state.file_path = get_file_path(file_path);
    parse_node(ast, engine, &mut state)?;
    Ok(())
}
///parse recursively from node root
fn parse_node(ast: &clang::Entity, engine: &mut Engine, state: &mut ParseState) -> anyhow::Result<()>{
    state.clear();
    match ast.get_kind() {
        clang::EntityKind::StructDecl => {
            if let Some(def) = ast.get_definition(){
                state.is_struct = true;
                parse_class(ast, state)?;
            }
        },
        clang::EntityKind::ClassDecl => {
            if let Some(def) = ast.get_definition(){
                state.is_class = true;
                parse_class(ast, state)?;
            }
        },
        clang::EntityKind::UnionDecl => todo!(),
        clang::EntityKind::EnumDecl => {
            if let Some(def) = ast.get_definition(){
                parse_enum(ast, state)?;
            }
        },
        clang::EntityKind::FunctionDecl => todo!(),
        clang::EntityKind::FieldDecl => todo!(),
        _ => {
            for child in ast.get_children() {
                parse_node(&child, engine, state)?;
            }
        }
    }
    Ok(())
}
///parse enum
/// int enum only
fn parse_enum(node: &clang::Entity, _state: &mut ParseState) -> anyhow::Result<Option<CppEnum>>{
    let Some(name) = node.get_name() else {return Ok(None)};
    let mut cenum= CppEnum{
        name: name,
        ..Default::default()
    };
    Ok(Some(cenum))
}
///parse class
fn parse_class(node: &clang::Entity, state: &mut ParseState) -> anyhow::Result<Option<UnrealClass>>{
    //empty class decl
    let Some(name) = node.get_name()else{
        return Ok(None);
    };
    let mut class = UnrealClass{
        is_struct: state.is_struct,
        name: name,
        ..Default::default()
    };
    for member in &node.get_children() {
        match member.get_kind() {
            clang::EntityKind::Method => {
                if let Some(mut api) = parse_api(member, state)?{
                    if api.is_public{
                        api.class_name = class.name.clone();
                        class.public_apis.push(api);
                    }
                    else{
                        //ignore
                    }
                }
            },
            clang::EntityKind::Constructor => {
                if let Some(mut api) = parse_api(member, state)?{
                    api.is_construstor = true;
                    if api.is_public{
                        api.class_name = class.name.clone();
                        class.public_apis.push(api);
                    }
                }
            }
            clang::EntityKind::Destructor => todo!(),
            clang::EntityKind::FieldDecl => {
                let field = parse_field(member, state)?;
                if field.is_public{
                    class.properties.push(field);
                }
                else{
                    class.none_public_properties.push(field);
                }
            },
            clang::EntityKind::UnionDecl => {
                parse_union(member, &mut class, state)?;
            },
            _ => (),
        }
    }
    Ok(Some(class))
}
//parse union to property at this moment
fn parse_union(node: &clang::Entity, class: &mut UnrealClass, state: &mut ParseState) -> anyhow::Result<()>{
    //only struct is supported
    if !class.is_struct{
        return Ok(());
    }
    //parse first stuct
    for child in node.get_children() {
        println!("union child {:?}", (
            child.get_name(),
            child.get_definition(),
            child.get_canonical_entity(),
            child.get_completion_string(),
            child.get_kind(),
            child.get_children().iter().map(|child| (child.get_name(), child.get_definition(), child.get_kind(),  child.get_enum_constant_value())).collect::<Vec<_>>()
        ))
    }
    Ok(())
}
///parse api
fn parse_api(node: &clang::Entity, state: &ParseState) -> anyhow::Result<Option<CppApi>>{
    println!(
        "\r\napi info {:?}",
        (
            node.get_name(),
            node.get_definition(),
            node.get_canonical_entity(),
            node.get_accessibility(),
            node.get_kind(),
            node.get_storage_class(),
            node.get_result_type(),
            node.get_arguments().as_ref().map(|args|{
                args.iter().map(|arg| (arg.get_name(), arg.get_kind(), arg.is_const_method(), arg.get_completion_string())).collect::<Vec<_>>()
            }),
            node.get_children().iter().map(|child| (child.get_name(), child.get_kind(), child.is_const_method(), child.get_completion_string())).collect::<Vec<_>>()
        )
    );
    let Some(method_name) = node.get_name()else{return Ok(None);};
    let Some(access) = node.get_accessibility()else{return Ok(None);};
    if method_name.starts_with("operator"){
        return Ok(None);
    }
    let mut api = CppApi{
        name: method_name,
        is_public: access == clang::Accessibility::Public,
        is_const: node.is_const_method(),
        ..Default::default()
    };
    // if state.file_path.contains("Object.h") && api.name == "IsBasedOnArchetype"{
    //     println!("pause");
    // }
    parse_parm_decl(&node.get_children(), &mut api, state)?;
    
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
fn parse_field(node: &clang::Entity, state: &ParseState) -> anyhow::Result<CppProperty>{
    println!(
        "\r\nfield info {:?}",
        (
            node.get_name(),
            node.get_range(),
            node.get_location(),
            node.get_definition(),
            node.get_kind(),
            node.get_storage_class(),
            node.get_result_type(),
            node.get_offset_of_field(),
            node.get_type().map(|tp|{
                (
                    tp.visit_fields(|field| {
                        println!("^^^^^^^field {:?}^^^^^^^", field.get_completion_string());
                        true
                    }),
                    tp.get_offsetof(node.get_name().unwrap()), 
                    tp.get_declaration(),
                    tp.get_typedef_name(),
                    tp.get_kind()
                )
            }),
            node.get_arguments().as_ref().map(|args|{
                args.iter().map(|arg| (arg.get_name(), arg.get_kind(), arg.is_const_method(), arg.get_completion_string(), arg.get_canonical_entity())).collect::<Vec<_>>()
            }),
            node.get_children().iter().map(|child| (child.get_name(), child.get_kind(), child.is_const_method(), child.get_completion_string(), child.get_canonical_entity())).collect::<Vec<_>>()
        )
    );
    let Some(field_name) = node.get_name()else{return Err(anyhow::anyhow!("field name not found"))};
    let mut field = CppProperty{
        name: field_name,                    
        ..Default::default()
    };
    
    Ok(field)
}
fn parse_parm_decl(inner: &Vec<clang::Entity>, api: &mut CppApi, state: &ParseState) -> anyhow::Result<()>{
    let mut params: Vec<Parameter> = Vec::with_capacity(inner.len());
    let mut arg_count: usize = 0;
    for node in inner {
        println!(
            "\r\nparam info {:?}",
            (
                node.get_kind(),
                node.get_availability(),
                node.get_range(),
                node.get_location(),
                node.get_type().map(|tp|{
                    (
                        tp.is_const_qualified(),
                        tp.get_modified_type(),
                        tp.get_elaborated_type(),
                        tp.get_class_type(), tp.get_pointee_type(), tp.get_ref_qualifier(), 
                        tp.get_kind(), tp.get_element_type(), tp.get_fields(), tp.get_result_type()
                    )
                }),
                node.get_mangled_names(),
                node.get_completion_string(),
                node.get_definition(),
                node.get_typedef_underlying_type(),
                node.get_canonical_entity(),
            )
        );
        if clang::EntityKind::ParmDecl == node.get_kind(){
            let mut param = Parameter::default();
            if let Some(name) = node.get_display_name(){
                param.name = name;                
            }
            param.const_param = !node.is_mutable();
            
        }
        else{
            panic!("undefined param {:?}", (node.get_name(), node.get_kind()));
        }
    }
    api.parameters = params;
    Ok(())
}
#[inline]
fn is_space(index: &str) -> bool {
    index == " " || index == "\t"
}