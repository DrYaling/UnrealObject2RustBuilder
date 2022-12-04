#![allow(non_snake_case)]
use std::{path::Path, str::EncodeUtf16, fs};

use serde::Deserialize;

use super::unreal_engine::{Engine, UnrealClass};
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
    pub storageClass: Option<String>,
    pub r#type: Option<QualType>,
    #[serde(default)]
    pub access: String,
    #[serde(default)]
    pub tagUsed: String,

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
    let mut command = std::process::Command::new("powershell");
    // command.arg("chcp 65001\r\n").output().ok();
    for file in files {
        let file_name = Path::new(&file).file_stem().unwrap().to_str().unwrap();
        let file_path = current_dir.join(&file).display().to_string().replace("/", "\\");
        let target_path = format!("{}\\{}.json", target_dir, file_name);
        // -encoding utf8
        let cmd = format!(
            "clang -Xclang -ast-dump=json -fsyntax-only -x c++ {}",
            file_path//, target_path
        );
        println!("cmd {}", cmd);
        let output = command
            // .arg("chcp 65001\r\n")
        .arg("/c")
        .arg(&cmd)
        // .arg(format!("\r\n out-file {} -encoding utf8", target_path))
        .output()?;
        // let result = unsafe{ String::from_utf8_unchecked(command
        // .arg(format!("out-file {} -encoding utf8", target_path))
        // .output()?.stderr)};
        // println!("out file {}", result);
        if !output.status.success(){
            println!("file {} cmd {} result {:?}", file_name, output.status, unsafe{ String::from_utf8_unchecked(output.stderr)});
        }
        let out_file = unsafe{ String::from_utf8_unchecked(output.stdout)};
        // let json_file = std::fs::read(&target_path)?;
        // // let ast: Node = serde_json::from_reader(std::fs::File::open(&target_path)?)?;
        // let json_str = if let Ok(s) = String::from_utf8(json_file.clone()){
        //     s
        // }
        // else if let Ok(s) = {
        //     assert!(json_file.len() % 2 == 0);
        //     let mut utf_16: Vec<u16> = Vec::with_capacity(json_file.len() / 2);
        //     unsafe{ std::intrinsics::copy(json_file.as_ptr(), utf_16.as_mut_ptr() as *mut u8, json_file.len() / 2) };
        //     String::from_utf16(utf_16.as_slice())
        // }{
        //     s
        // }
        // else{
        //     println!("json file {target_path} is not utf8 or utf16");
        //     continue;
        // };
        // println!("out_file {}", out_file);
        let ast: Node = serde_json::from_str(&out_file)?;
        std::fs::write(target_path, out_file)?;
        println!("cmd ast {}", ast.id.to_string());
        parse_file(&ast, &file_path, engine)?; 
        break;
    }    
    Ok(())
}
#[derive(Debug, Default)]
struct ParseState{
    is_pub: bool,
    is_struct: bool,
    is_class: bool,  
    content: String,     
}
impl ParseState{
    fn clear(&mut self){
        self.is_class = false;
        self.is_struct = false;
        self.is_pub = false;            
    }
}
///parse file with ast
fn parse_file(ast: &Node, file_path: &str, engine: &mut Engine) -> anyhow::Result<()>{

    let mut state = ParseState::default();
    state.content = std::fs::read_to_string(file_path)?;
    parse_node(ast, engine, &mut state)?;
    Ok(())
}
fn parse_node(ast: &Node, engine: &mut Engine, state: &mut ParseState) -> anyhow::Result<()>{
    match ast.kind.kind {
        clang_ast::Kind::AccessSpecDecl => {
            state.is_pub = ast.kind.access == "public";
        },
        clang_ast::Kind::CXXConstructorDecl => {

        },
        clang_ast::Kind::CXXConversionDecl => todo!(),
        clang_ast::Kind::CXXDestructorDecl => todo!(),
        clang_ast::Kind::CXXMethodDecl => todo!(),
        clang_ast::Kind::CXXRecordDecl => {
            let kind = &ast.kind;
            match kind.tagUsed.as_str() {
                "class" => {

                },
                "struct" => {

                }
                _ => (),
            }
        },
        clang_ast::Kind::DecltypeType => todo!(),
        clang_ast::Kind::DecompositionDecl => todo!(),
        clang_ast::Kind::EmptyDecl => todo!(),
        clang_ast::Kind::EnumConstantDecl => todo!(),
        clang_ast::Kind::EnumDecl => todo!(),
        clang_ast::Kind::FunctionDecl => todo!(),
        clang_ast::Kind::NamespaceDecl => todo!(),
        clang_ast::Kind::ParenType => todo!(),
        clang_ast::Kind::ParmVarDecl => todo!(),
        clang_ast::Kind::TypedefDecl => todo!(),
        clang_ast::Kind::UsingDecl => todo!(),
        clang_ast::Kind::VarDecl => todo!(),
        _ => (),
    }
    Ok(())
}
fn parse_class(inner: &Vec<Node>, state: &mut ParseState) -> anyhow::Result<Option<UnrealClass>>{
    for node in inner {
        let kind = &node.kind; 
        match kind.kind {
            clang_ast::Kind::AccessSpecDecl => {
                state.is_pub = kind.access == "public";
            },
            clang_ast::Kind::CXXConstructorDecl => {

            },
            clang_ast::Kind::CXXConversionDecl => todo!(),
            clang_ast::Kind::CXXDestructorDecl => todo!(),
            clang_ast::Kind::CXXMethodDecl => todo!(),
            clang_ast::Kind::CXXRecordDecl => {
                let kind = &ast.kind;
                match kind.tagUsed.as_str() {
                    "class" => {

                    },
                    "struct" => {
                        
                    }
                    _ => (),
                }
            },
            clang_ast::Kind::DecltypeType => todo!(),
            clang_ast::Kind::DecompositionDecl => todo!(),
            clang_ast::Kind::EmptyDecl => todo!(),
            clang_ast::Kind::EnumConstantDecl => todo!(),
            clang_ast::Kind::EnumDecl => todo!(),
            clang_ast::Kind::FunctionDecl => todo!(),
            clang_ast::Kind::NamespaceDecl => todo!(),
            clang_ast::Kind::ParenType => todo!(),
            clang_ast::Kind::ParmVarDecl => todo!(),
            clang_ast::Kind::TypedefDecl => todo!(),
            clang_ast::Kind::UsingDecl => todo!(),
            clang_ast::Kind::VarDecl => todo!(),
            _ => (),
        }
    }
    Ok(None)
}