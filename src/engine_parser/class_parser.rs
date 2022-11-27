use super::{unreal_engine::Engine, config::*};
use std::{sync::Mutex, fs::DirEntry};
use once_cell::sync::Lazy;
use std::path::Path;
static ENGINE_CODE: Lazy<Mutex<Engine>> = Lazy::new(|| Mutex::new(Default::default()));
///parse classes
pub fn parse(settings: CustomSettings) -> anyhow::Result<ExportDetails>{
    let engine_root = Path::new(&settings.EngineRoot);
    let runtime_root = engine_root.join("Source/Runtime/");
    let engine_code = crate::read_files(&runtime_root.as_path().display().to_string(), ".h")?;
    let mut engine = Engine::default();
    std::fs::create_dir("engine_code").ok();
    for file in engine_code{
        match std::fs::read_to_string(&file) {
            Ok(fc) => {
                parse_header(&mut engine, file, fc)?;
            }
            Err(e) => {
                    println!("file  {file} read fail {:?}", e);
                }
        }
    }
    for class in &settings.ExportClasses{

    }
    Ok(Default::default())
}
///parse header struct
fn parse_header(engine: &mut Engine, file: String, content: String) -> anyhow::Result<()>{
    let file_name = Path::new(&file).file_name().unwrap().to_str().unwrap();
    // std::fs::write(format!("engine_source/{}", file_name), &content)?;
    if file_name != "Actor.h" {//} && !file.contains("Object.h"){
        return Ok(());
    }
    let mut lines = content.split("\n").map(|s| s.to_owned()).collect::<Vec<_>>();
    remove_comment_and_include(&mut lines);
    remove_unreal_tags(&mut lines);
    remove_deprecated(&mut lines);
    remove_defines(&mut lines);
    remove_templates(&mut lines);
    std::fs::write(format!("engine_code/{}_trip", file_name), lines.join("\r\n"))?;
    remove_none_public(&mut lines);
    // let mut read_line = 0usize;
    // while read_line < lines.len() {
        
    // }
    if lines.len() == 0{
        return Ok(());
    }
    std::fs::write(format!("engine_code/{}", file_name), lines.join("\r\n"))?;
    engine.files.push(file.clone());
    engine.file_caches.insert(file, content);
    Ok(())
}
///移除注释
fn remove_comment_and_include(lines: &mut Vec<String>){
    enum ReadState{
        Normal,
        Comment,
    }
    let mut read_line: usize = 0;
    let mut state = ReadState::Normal;
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_string();
        match state {
            ReadState::Normal => {
                if line.is_empty() || line.starts_with("//") || (line.starts_with("#") && !line.starts_with("#define")){
                    lines.remove(read_line);
                    continue;
                }
                else if let Some(index) = line.find("//"){
                    lines[read_line] = line[0..index].to_string();
                }
                //comment with /*
                else if line.contains("/*"){
                    state = ReadState::Comment;
                    continue;
                }
            },
            ReadState::Comment => {
                let mut start_index = usize::MAX;
                if let Some(idx) = line.find("/*"){
                    start_index = idx;
                }
                if line.ends_with("*/"){
                    state = ReadState::Normal;
                    if start_index != usize::MAX{
                        lines[read_line] = line[0..start_index].to_string();
                    }
                    else{
                        lines.remove(read_line);    
                        continue;  
                    }
                }
                else if let Some(com_end) = line.find("*/"){
                    state = ReadState::Normal;
                    if start_index != usize::MAX{
                        lines[read_line] = line[0..start_index].to_string() + &line[com_end..].to_string();   
                    }
                    else{
                        lines[read_line] = line[com_end + 1..].to_string();   
                    }  
                }
                else{
                    if start_index != usize::MAX{
                        lines[read_line] = line[0..start_index].to_string();
                    }
                    else{
                        lines.remove(read_line);    
                        continue;  
                    }
                }
            },
        }
        read_line += 1;
    }
}
///移除泛型
fn remove_templates(lines: &mut Vec<String>){    
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_owned();
        if line.contains("template"){
            let mut brace_count = 0;
            //remove first brace
            while read_line < lines.len() {
                let rl = read_line;
                let line = lines.remove(rl);
                if line.contains("{"){
                    break;
                }
            }
            while read_line < lines.len() {
                let line = lines.remove(read_line);
                if line.contains("{"){
                    brace_count += 1;
                }
                if line.contains("}"){
                    //template class ignored
                    if brace_count == 0{
                        break;
                    }
                    brace_count -= 1;
                }
            }
            continue;
        }
        read_line += 1;
    }
}
///移除标记宏
fn remove_unreal_tags(lines: &mut Vec<String>){
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_owned();
        if line.contains("UENUM") ||
            line.contains("UCLASS") ||
            line.contains("DEFINE_ACTORDESC_TYPE") ||
            line.contains("DECLARE_DELEGATE_SixParams") ||
            line.contains("DECLARE_EVENT_TwoParams") ||
            line.contains("DECLARE_DELEGATE_RetVal_ThreeParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_OneParam") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_TwoParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_ThreeParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_FourParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_FiveParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_SixParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_SevenParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_EightParams") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_SPARSE_DELEGATE_NineParams") ||
            line.contains("USTRUCT") ||
            line.contains("DECLARE_CLASS") ||
            line.contains("UFUNCTION") ||
            line.contains("DEFINE_DEFAULT_OBJECT_INITIALIZER_CONSTRUCTOR_CALL") ||
            line.contains("GENERATED_BODY") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_DELEGATE") ||
            line.contains("DECLARE_LOG_CATEGORY_EXTERN") ||
            line.contains("GENERATED_USTRUCT_BODY") ||
            line.contains("UPROPERTY"){
                lines.remove(read_line);
                continue;
        }
        let sps = line.split(" ").map(|s| s.to_string()).collect::<Vec<_>>();
        let mut empty = line.is_empty();
        for key in sps {
            if key.contains("_API") ||
                key.to_lowercase().contains("inline"){
                lines[read_line] = lines[read_line].replace(&key, "");
                empty |= lines[read_line].trim().is_empty();
            }
        }
        if empty{
            lines.remove(read_line);
            continue;
        }
        read_line += 1;
    }
}
fn remove_deprecated(lines: &mut Vec<String>){    
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_owned();
        if line.starts_with("UE_DEPRECATED"){
            lines.remove(read_line);
            let start = read_line;
            parse_function(lines, &mut read_line);
            (start..=read_line).into_iter().for_each(|index|{
                if lines.len() > index{
                    lines.remove(index);
                }
            });
        }
        read_line += 1;
    }
}
fn remove_defines(lines: &mut Vec<String>){
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_owned();
        if line.starts_with("#define"){
            while read_line < lines.len() {
                let line = lines.remove(read_line).trim().to_string();
                if !line.ends_with("\\"){
                    break;
                }
            }
            continue;
        }
        read_line += 1;
    }
}
///移除class的非公共接口
fn remove_none_public(lines: &mut Vec<String>){
    enum ClassFieldState{
        Public, //public block
        NonePub,//not public
    }
    
    enum ClassBraceState{
        Start,
        Between,
        End,
    }
    enum ClassBlockState{
        NotIn,
        In(ClassBraceState),
    }
    let mut state = ClassFieldState::NonePub;
    let mut class_state = ClassBlockState::NotIn;
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let mut line = lines[read_line].trim().to_owned();
        if line.contains("SetInstigator"){
            println!("line {}", line);
        }
        match &mut class_state {
            ClassBlockState::NotIn => {
                if line.starts_with("class ") && !line.ends_with(";"){
                    state = ClassFieldState::NonePub;
                    //class start
                    if line.ends_with("{"){                        
                        class_state = ClassBlockState::In(ClassBraceState::Between);
                        state = ClassFieldState::NonePub;
                    }
                    else{
                        class_state = ClassBlockState::In(ClassBraceState::Start);
                    }
                }
            },
            ClassBlockState::In(b_state) => {
                match b_state {
                    //class start
                    ClassBraceState::Start => {
                        if line.starts_with("{"){
                            *b_state = ClassBraceState::Between;
                            state = ClassFieldState::NonePub;
                        }
                    },
                    //class block
                    ClassBraceState::Between => {
                        if line.starts_with("public:"){
                            state = ClassFieldState::Public;
                            line = line.replace("public:", "").trim().to_string();
                        }
                        else if line.starts_with("private:") || line.starts_with("protected:"){
                            state = ClassFieldState::NonePub;
                            line = line.replace("private:", "").replace("protected:", "").trim().to_string();
                        }
                        if !line.is_empty(){
                            match state {
                                //remove none public properties
                                ClassFieldState::NonePub => {
                                    //ignore none public members
                                    let start_line = read_line;
                                    let (member,end) = parse_class_member(lines, &mut read_line);
                                    if member.is_some(){
                                        (start_line..read_line+1).into_iter().for_each(|idx| 
                                            if idx < lines.len(){
                                                lines.remove(idx);
                                            }
                                        );
                                    }
                                    if end{
                                        let end_line = &lines[read_line - 1];
                                        let end_line1 = &lines[read_line - 2];
                                        let end_line2 = &lines[read_line - 3];
                                        class_state = ClassBlockState::NotIn;
                                        if member.is_some(){
                                            lines.push("}".to_string());
                                        }
                                    }
                                    continue;
                                },
                                ClassFieldState::Public => {
                                    let (member, end) = parse_class_member(lines, &mut read_line);
                                    //keep menbers
                                    if let Some(member) = &member{
                                        match member {
                                            ClassMember::Property(_) => {},
                                            ClassMember::Function(_) => (),
                                            ClassMember::Empty => (),
                                            ClassMember::Brace(_) => (),
                                        }
                                    }
                                    if end{
                                        let end_line = &lines[read_line - 1];
                                        let end_line1 = &lines[read_line - 2];
                                        let end_line2 = &lines[read_line - 3];
                                        class_state = ClassBlockState::NotIn;
                                        if member.is_some(){
                                            lines.push("}".to_string());
                                        }
                                    }
                                },
                            }
                        }
                        else {
                            lines.remove(read_line);
                            continue;
                        }

                    },
                    //end
                    ClassBraceState::End => (),
                }
            },
        }
        read_line += 1;
    }
}
enum ClassMember {
    Property(CppProperty),
    Function(CppApi),
    Brace(BraceType),
    Empty,
}
///(int field)
struct Brackets{
    content: Option<String>,
}
///分析类成员信息，并返回类是否结束
fn parse_class_member(lines: &mut Vec<String>, read_line: &mut usize) -> (Option<ClassMember>, bool){
    if ignore_class_or_struct(lines, read_line){
        *read_line -= 1;
        return (None, false);
    }
    if let Some(function) = parse_function(lines, read_line){
        return (Some(ClassMember::Function(function)), false);
    }
    else if let Some(property) = parse_field(lines, read_line) {
        return (Some(ClassMember::Property(property)), false);
    }
    else if let Some(enum_type) = parse_enum(lines, read_line){
        return (Some(ClassMember::Property(Default::default())), false);
    }
    match parse_remove_brace(lines, read_line, true) {
        Some(brace) => {
            match brace {
                BraceType::Start => return (Some(ClassMember::Brace(brace)), false),
                BraceType::End => return (Some(ClassMember::Brace(brace)), true),
                BraceType::StartAndEnd(content) => panic!("unknown brace {content} at line {read_line}"),
            }
        },
        None => {
            (None, false)
        },
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum  BraceType {
    Start,
    End,
    StartAndEnd(String),
}
//移除并返回花括号
fn parse_remove_brace(lines: &mut Vec<String>, read_line: &mut usize, remove: bool) -> Option<BraceType>{
    let line = lines[*read_line].trim().to_string();
    let start = line.find("{");
    let end = line.find("}");
    let ret = match (start, end) {
        (Some(start), None) => {
            if remove{
                lines[*read_line] = line[0..start].to_string();
            }
            Some(BraceType::Start)
        },
        (Some(start), Some(end)) => {
            if remove{
                lines[*read_line] = line[0..start].to_string();
                if end < line.len(){
                    lines[*read_line].push_str(&line[end+1..].to_string());
                }
            }
            Some(BraceType::StartAndEnd(line[start..end].to_string()))
        },
        (None, Some(end)) => {
            if !line.ends_with("};") &&
                end != line.len() - 1{
                println!("end content maybe error ignored in the end of {{{}", line[end..].to_string());
            }
            if remove{
                lines[*read_line] = line[0..end].to_string();
            }
            Some(BraceType::End)
        }
        (None, None) => None
    };    
    if lines[*read_line].is_empty(){
        lines.remove(*read_line);
    }
    ret
}
fn parse_brace_line(lines: &mut Vec<String>, read_line: &mut usize) -> Option<BraceType>{
    let line = lines[*read_line].trim();
    if line == "}"{
        Some(BraceType::End)
    }
    else if line == "{"{
        Some(BraceType::Start)
    }
    else if line == "{}"{        
        Some(BraceType::StartAndEnd(String::default()))
    }
    else{
        None
    }
}
///解析函数
fn parse_function(lines: &mut Vec<String>, read_line: &mut usize) -> Option<CppApi>{
    let mut line = lines[*read_line].trim().to_string();
    let start = line.find("(");
    let end = line.find(")");
    match (start, end) {
        (Some(start), Some(end)) => {
            //function defination
            let mut override_flag = false;
            if let Some(index) = line.find("override"){
                line = line.replace("override", "").trim().to_string();
                override_flag = true
            }
            let parameters = line[start + 1..end]
            .split(",")
            .map(|s| {
                let parameter = s.split(" ").map(|p| p.trim().to_string()).collect::<Vec<_>>();
                parameter
            })
            .collect::<Vec<_>>();
            let function_impl =  line[0..start]
            .split(" ").map(|sa| sa.to_string())
            .collect::<Vec<_>>();
            ///function ended
            fn ended(l: &str, _end: usize) -> bool{
                let l = l.trim_end();
                return l.ends_with(";") || l.ends_with("}");
            }
            let mut ended = line.ends_with(";") || line.ends_with("}");
            //ignore content
            while !ended {
                match parse_remove_brace(lines, read_line, false) {
                    Some(brace) => {
                        match brace {
                            BraceType::Start => *read_line += 1,
                            BraceType::End => ended = true,
                            BraceType::StartAndEnd(_) => ended = true,
                        }
                    },
                    None => *read_line += 1,
                }
            }
            // *read_line += 1;
            Some(CppApi::default())
        }
        _ => None,
    }
}
fn parse_field(lines: &mut Vec<String>, read_line: &mut usize) -> Option<CppProperty>{
    let line = lines[*read_line].trim().to_string();
    if line.contains(";"){
        Some(CppProperty::default())
    }
    else{
        None
    }
}
///解析枚举
fn parse_enum(lines: &mut Vec<String>, read_line: &mut usize) -> Option<CppEnum>{
    if lines[*read_line].contains("enum"){
        let line = lines[*read_line].trim().replace("enum ", "").replace("class ", "").to_string();
        while !lines[*read_line].contains("}") {
            *read_line += 1;
        }
        *read_line += 1;
        Some(CppEnum::default())
    }
    else{
        None
    }
}
///忽略代码中的局部类和结构体(class 内部)
fn ignore_class_or_struct(lines: &mut Vec<String>, read_line: &mut usize) -> bool{
    let l = lines[*read_line].trim();
    if l.starts_with("class") || l.starts_with("struct"){
        let mut brace = 0;
        while *read_line < lines.len() {
            let line = lines.remove(*read_line).trim().to_string();
            //impliment
            if line.ends_with(";"){
                return true;
            }
            //type define
            else if line.ends_with("{"){
                break;
            }
        }
        ignore_class_or_struct(lines, read_line);
        while *read_line < lines.len() {
            let line = lines.remove(*read_line).trim().to_string();
            if line.ends_with("{"){
                if brace == 0{
                    break;
                }
                brace -= 1;
            }
            else if line.ends_with("}") || line.ends_with("};"){
                brace += 1;
            }
        }
        true
    }
    else{
        false
    }
}