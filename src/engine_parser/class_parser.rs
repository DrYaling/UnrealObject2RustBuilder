use crate::engine_parser::unreal_engine::UnrealClass;

use super::{unreal_engine::Engine, config::*};
use std::{path::Path, sync::atomic::{AtomicUsize, Ordering}};
// static ENGINE_CODE: Lazy<Mutex<Engine>> = Lazy::new(|| Mutex::new(Default::default()));
const fn split_space(c: char) -> bool{
    c == ' ' || c == '\t'
}
static mut RUNTIME_ROOT: String = String::new();
#[inline]
pub fn runtime_root() -> String{
    unsafe{
        RUNTIME_ROOT.replace("\\", "/")
    }
}
///parse classes
pub fn  parse(settings: CustomSettings) -> anyhow::Result<ExportDetails>{
    let engine_root = Path::new(&settings.EngineRoot);
    let runtime_root = engine_root.join("Source/Runtime/");
    unsafe{ 
        RUNTIME_ROOT = runtime_root.display().to_string();
    }
    let engine_json_path = "configs/engine.json";
    let engine_code = crate::read_files(&runtime_root.display().to_string(), ".h")?;
    let engine = if let Ok(engine_json) = std::fs::read_to_string(engine_json_path){
        serde_json::from_str(&engine_json)?
    }
    else{
        let mut engine = Engine::default();
        std::fs::create_dir("engine_code").ok();
        let ignores = settings.IgnoreFiles.clone();
        let ignore = |file: &str| -> bool{
            //path in export config
            if settings.ExportPathRoot.iter().find(|root| file.contains(root.as_str())).is_none(){
                return true;
            }
            let file_name = Path::new(file)
            .file_name()
            .expect(&format!("{file} is not a file"))
            .to_str().unwrap()
            .split(".")
            .next()
            .unwrap()
            .to_string();
    
            for ignore in &ignores {
                if ignore.starts_with("*") && ignore.ends_with("*"){
                    if file_name.contains(ignore){
                        return true;
                    }
                }
                else if ignore.starts_with("*"){
                    let ignore = &ignore[1..];
                    if file_name.ends_with(ignore){
                        return true;
                    }
                }
                else if ignore.ends_with("*"){
                    let ignore = &ignore[0..ignore.len() - 1];
                    if file_name.starts_with(ignore){
                        return true;
                    }
                }
                else if &file_name == ignore{
                    return true;
                }
            }
            false
        };
        for file in engine_code{
            if ignore(&file){
                continue;
            }
            match std::fs::read_to_string(&file) {
                Ok(fc) => {
                    parse_header(&mut engine, file, fc)?;
                }
                Err(e) => {
                        println!("file  {file} read fail {:?}", e);
                    }
            }
        }
        // println!(
        //     "class {:?}", 
        //     engine.classes.first().map(|class| (class.public_apis.last(), class.properties.last()))
        // );
        super::ast::run(&mut engine)?;
        //TODO
        // let engine_str = serde_json::to_string(&engine)?;
        // std::fs::write(engine_json_path, engine_str)?;
        engine
    };
    super::bindgen::generate(&engine, &settings)?;
    Ok(Default::default())
}
static FAIL_COUNT: AtomicUsize = AtomicUsize::new(0);
///parse header struct
fn parse_header(engine: &mut Engine, file: String, content: String) -> anyhow::Result<()>{
    let file_name = Path::new(&file).file_name().unwrap().to_str().unwrap();
    // std::fs::write(format!("engine_source/{}", file_name), &content)?;
    // if file_name != "Actor.h" {
    //     return Ok(());
    // }
    // println!("parse file {}", file_name);
    let mut lines = content.replace("\r", "").split("\n").map(|s| s.to_owned()).collect::<Vec<_>>();
    if let Ok(content) = std::fs::read_to_string(format!("engine_code/{}", file_name)){
       lines = content.split("\r\n").map(|s| s.to_owned()).collect::<Vec<_>>();
    }
    else 
    {
        let inner_lines = std::mem::take(&mut lines);
        match std::panic::catch_unwind(||{
                    let mut inner_lines = inner_lines;
                    remove_unreal_tags(&mut inner_lines);
                    // std::fs::write(format!("engine_code/{}_rem_utag", file_name), lines.join("\r\n"))?;
                    remove_comment_and_mic(&mut inner_lines);
                    // // std::fs::write(format!("engine_code/{}_remove_include", file_name), lines.join("\r\n"))?;
                    remove_defines(&mut inner_lines);
                    // std::fs::write(format!("engine_code/{}_rem_def", file_name), lines.join("\r\n"))?;
                    remove_templates(&mut inner_lines);
                    // std::fs::write(format!("engine_code/{}_rem_temp", file_name), lines.join("\r\n"))?;
                    remove_deprecated(&mut inner_lines);
                    // std::fs::write(format!("engine_code/{}_rem_dep", file_name), lines.join("\r\n"))?;
                    remove_all_fn_block(&mut inner_lines);
                    //normalize_all(&mut inner_lines);
                    inner_lines
                }) {
            Err(_e) => {
                // println!("parse flush file {} fail {:?}", file_name, e);
                FAIL_COUNT.fetch_add(1, Ordering::Release);
                println!("file {file_name} parse flush fail >>>>fail count {}", FAIL_COUNT.load(Ordering::Relaxed));
                return Ok(());
            }
            Ok(clines) => lines = clines,
        }
        // std::fs::write(format!("engine_code/{}_trip", file_name), lines.join("\r\n"))?;
    }    
    // load_public_exports(&mut lines, engine, &file);
    if lines.len() == 0{
        return Ok(());
    }
    let catch_lines = std::mem::take(&mut lines);
    match std::panic::catch_unwind(||{
            let catch_lines = catch_lines;
            let engine = Engine::default();
            // load_public_exports(&mut catch_lines, &mut engine, &file);
            (catch_lines, engine)
        }) {
        Ok((clines, mut cengine)) => {
            engine.classes.append(&mut cengine.classes);
            engine.enums.append(&mut cengine.enums);
            engine.static_apis.append(&mut cengine.static_apis);
            engine.value_types.append(&mut cengine.value_types);
            engine.files.push(file.clone());
            lines = clines;
            let root = runtime_root();
            engine.file_paths.insert(file_name.to_string(), file.replace(&root, ""));
            let file_path = format!("engine_code/{}", file.replace(&root, ""));
            std::fs::create_dir_all(&Path::new(&file_path).parent().unwrap()).ok();
            std::fs::write(&file_path, lines.join("\r\n"))?;
            return Ok(());
        }
        Err(_e) => {
            FAIL_COUNT.fetch_add(1, Ordering::Release);
            println!("file {file_name} parse fail >>>>fail count {}", FAIL_COUNT.load(Ordering::Relaxed));
        },
    }
    // println!("file {} parse fail", file_name);
    Ok(())
}
///移除注释
fn remove_comment_and_mic(lines: &mut Vec<String>){
    enum ReadState{
        Normal,
        Comment,
    }
    let mut read_line: usize = 0;
    let mut state = ReadState::Normal;
    fn type_impl(line: &str) -> bool{
        (
            line.starts_with("typedef ") ||
            line.starts_with("struct ") ||
            line.starts_with("class ") ||
            line.starts_with("friend class ")
        )
        && line.contains(";")
    }
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_string();
        match state {
            ReadState::Normal => {
                if line.is_empty() || 
                    line.starts_with("//") || 
                    line.starts_with("using ") ||
                    (line.starts_with("#") && !line.starts_with("#define")) |
                    type_impl(&line){
                    lines.remove(read_line);
                    continue;
                }
                else if let Some(index) = line.find("//"){
                    lines[read_line] = line[0..index].to_string();
                }
                else if line.starts_with("enum class"){
                    lines[read_line] = lines[read_line].replace("enum class", "enum");
                }
                //comment with /*
                else if line.contains("/*"){
                    state = ReadState::Comment;
                    continue;
                }
                //union
                // else if let Some(idx) = line.find("union"){
                //     //left space or right space
                //     if idx > 0 && idx < line.len() - 5{
                //         if is_space(&line[idx -1..idx]) ||
                //             is_space(&line[idx + 1..=idx + 1]){
                //             remove_fn_block(lines, &mut read_line);
                //             lines.remove(read_line);
                //             continue;   
                //         }
                //     }
                //     else if idx == 0{
                //         if line.len() == 5 || is_space(&line[idx + 1..=idx + 1]){
                //             remove_fn_block(lines, &mut read_line);
                //             lines.remove(read_line);
                //             continue;   
                //         }
                //     }
                // }
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
        if line.starts_with("#define"){
            read_line += 1;
            continue;
        }
        if line.contains("UENUM") ||
            line.contains("UCLASS") ||
            line.contains("UPROPERTY") ||
            line.contains("DEPRECATED_") ||
            line.contains("UINTERFACE") ||
            line.contains("DEFINE_ACTORDESC_TYPE") ||
            line.contains("DECLARE_") ||
            line.contains("PRAGMA_") ||
            line.contains("BINDING_") ||
            line.contains("PROPERTY_BINDING_") ||
            line.contains("STAT") ||
            line.contains("SHADER_") ||
            line.contains("LAYOUT_") ||
            line.contains("_HIDE") ||
            line.contains("HIDE_") ||
            line.contains("JSON_") ||
            line.contains("IMPL_") ||
            line.contains("IMPLEMENT_") ||
            line.contains("UE_NONCOPYABLE") ||
            line.contains("UE_TRACE") ||
            line.contains("UE_LOG") ||
            line.contains("VARARGS") ||
            line.contains("DEFINE_") ||
            line.contains("SEQUENCER_INSTANCE_PLAYER_TYPE") ||
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
            line.contains("EFFECT_PRESET_METHODS") ||
            line.contains("DEFINE_DEFAULT_OBJECT_INITIALIZER_CONSTRUCTOR_CALL") ||
            line.contains("GENERATED_") ||
            line.contains("DECLARE_DYNAMIC_MULTICAST_DELEGATE") ||
            line.contains("DECLARE_LOG_CATEGORY_EXTERN") ||
            line.contains("GENERATED_USTRUCT_BODY") ||
            line.contains("UPROPERTY"){
                lines.remove(read_line);
                continue;
        }
        let sps = line.split(split_space).map(|s| s.to_string()).collect::<Vec<_>>();
        let mut empty = line.is_empty();
        for key in sps {
            if key.contains("_API") ||
                key.contains("STD_") ||
                key.contains("MS_ALIGN") ||
                key.contains("GCC_ALIGN") ||
                key.contains("STDCAll") ||
                key.to_lowercase().contains("inline"){
                lines[read_line] = lines[read_line].replace(&key, "");
                empty |= lines[read_line].trim().is_empty();
            }
        }
        if let Some(index) = line.find("UMETA("){
            let sub = &line[index..];
            if let Some(end) = sub.rfind(")"){
                let umeta_info = &line[index..=end + index];
                lines[read_line] = lines[read_line].replace(umeta_info, "");
            }
        }
        if line.starts_with("#if UE_EDITOR") || line.starts_with("#if WITH_EDITOR"){
            lines[read_line].clear();
            read_line += 1;
            let mut endif = 0;
            while read_line < lines.len() {
                let removed = lines.remove(read_line);
                if removed.starts_with("#if"){
                    endif += 1;
                }
                if removed.starts_with("#endif"){
                    if endif == 0{//paire with #if UE_EDITOR
                        break;
                    }
                    endif -= 1;
                }
            }
        }
        if empty{
            lines.remove(read_line);
            continue;
        }
        read_line += 1;
    }
}
fn remove_all_fn_block(lines: &mut Vec<String>){
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let check_line = lines[read_line].trim();
        if check_line.contains("CopyTexture(FTexture2DRHIRef"){
            println!("{}", check_line);
        }
        if check_line.starts_with("virtual") && (
            check_line.ends_with("=0;") ||
            check_line.ends_with("= 0;") ||
            check_line.ends_with("= 0 ;")
        ){
            //if end with = 0;
            lines.remove(read_line);
            continue;
        }
        //not function, skip
        if !normalize_function(lines, &mut read_line){
            read_line += 1;
            continue;
        }
        //panic if fail
        let line = lines[read_line].trim();
        let bracket_start = line.find("(");
        let bracket_end = line.find(")");
        let end_tag = line.find(";");
        let brace_end = line.find("}");
        match (bracket_start, bracket_end, end_tag, brace_end) {
            (Some(_start), Some(_end), None, None) |
            (Some(_start), Some(_end), Some(_), Some(_))=> {
                remove_fn_block(lines, &mut read_line);
            }
            _ => (),
        }
        //remove fn define and operator
        if lines[read_line].contains("::") || 
            lines[read_line].contains("operator") ||
            lines[read_line].starts_with("extern "){
            lines.remove(read_line);
            continue;
        }
        read_line += 1;
    }
}
//move function parameters into one line
fn normalize_function(lines: &mut Vec<String>, read_line: &mut usize) -> bool{
    if *read_line >= lines.len(){
        return false;
    }
    let normal_line = lines[*read_line].as_str().trim();
    if normal_line.starts_with("public:") ||
        normal_line.starts_with("private:") ||
        normal_line.starts_with("protected:"){
        return false;
    }
    if normal_line.contains("SeparateRawDataIntoTracks"){
        println!("is this");
    }
    let mut start_brace = lines[*read_line].find("(");
    let end_brace = lines[*read_line].find(")");
    if start_brace.is_some() && end_brace.is_some(){
        return true;
    }
    let end_flag = lines[*read_line].find(";");
    if end_flag.is_some(){
        return false;
    }
    //check if this is function
    let brack_end_line;
    'root: {
        let mut valid_check = *read_line + 1;
        let mut this_line;
        while valid_check < lines.len() {
            this_line = lines[valid_check].as_str();
            if let Some(_) = this_line.find(")"){
                if start_brace.is_some(){
                    brack_end_line = valid_check;
                    break 'root;
                }
                //with no function start bracket
                else{
                    return false;
                }
            }
            //end with no )
            else if let Some(_) = this_line.find(";"){
                //not function
                return false;
            }
            else if let Some(idx) = this_line.find("("){
                if start_brace.is_some(){
                    //error style with 2 (
                    return false;
                }
                else{
                    start_brace = Some(idx);
                }
            }
            valid_check += 1;
        }
        //check fail
        return false;
    }
    let mut parameters = vec![];
    let index = *read_line + 1;
    for _ in index..=brack_end_line {
        parameters.push(lines.remove(index));
    }
    lines[*read_line].push_str(&parameters.join(" "));
    true
    
}
fn remove_deprecated(lines: &mut Vec<String>){    
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let line = lines[read_line].trim();
        if line.starts_with("UE_DEPRECATED"){
            // let removed = 
            lines.remove(read_line);
            //remove fn brackets
            normalize_function(lines, &mut read_line);
            remove_fn_block(lines, &mut read_line);
            if read_line < lines.len(){
                // let _removed = 
                lines.remove(read_line);
                // println!("remove deprecated {removed}:{}", _removed);
            }
            continue;
            // let start = read_line;
            //parse_function(lines, &mut read_line);
            // (start..=read_line).into_iter().for_each(|index|{
            //     if lines.len() > index{
            //         lines.remove(index);
            //     }
            // });
        }
        read_line += 1;
    }
}
fn remove_defines(lines: &mut Vec<String>){
    let mut read_line = 0usize;
    while read_line < lines.len() {
        let line = lines[read_line].trim().to_owned();
        if line.starts_with("#define"){
            let mut _remove_line;
            while read_line < lines.len() {
                _remove_line = lines.remove(read_line).trim().to_string();
                if !_remove_line.ends_with("\\"){
                    break;
                }
            }
            continue;
        }
        read_line += 1;
    }
}
#[allow(unused)]
///解析class的公共接口
fn load_public_exports(lines: &mut Vec<String>, engine: &mut Engine, file_path: &str){
    enum ClassFieldState{
        Public, //public block
        NonePub,//not public
    }
    
    enum ClassBraceState{
        Start,
        Between,
        #[allow(unused)]
        End,
    }
    enum ClassBlockState{
        NotIn,
        In(ClassBraceState),
    }
    let mut state = ClassFieldState::NonePub;
    let mut class_state = ClassBlockState::NotIn;
    let mut read_line = 0usize;
    let mut class_info = UnrealClass::default();
    let mut last_line: String = "".into();
    while read_line < lines.len() {
        let mut line = lines[read_line].trim().to_owned();
        if line == last_line{
            // lines[read_line].push_str("UNKNOWN");
            read_line += 1;
            continue;
        }
        last_line = line.clone();
        if line.starts_with("FCachedAudioStreamingManagerParams"){
            println!("line {}", line);
        }
        match &mut class_state {
            ClassBlockState::NotIn => {
                let is_struct = line.starts_with("struct");
                let is_class = line.starts_with("class");
                if (is_class || is_struct) && !line.ends_with(";"){
                    state = if is_class{ClassFieldState::NonePub}else{ClassFieldState::Public};
                    let mut iter = line.split(split_space).filter(|s| !s.is_empty());
                    iter.next();
                    class_info.name = iter.next().map(|s| s.to_string()).expect("class or struct name expected").trim().replace(":", "");
                    class_info.is_struct = is_struct;
                    // class_info.path = file_path.to_string();
                    //class_info.
                    //class start
                    if line.ends_with("{"){                        
                        class_state = ClassBlockState::In(ClassBraceState::Between);
                        // state = ClassFieldState::NonePub;
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
                            // state = ClassFieldState::NonePub;
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
                                    let (member,end) = parse_class_member(lines, &mut read_line, Some(&class_info.name));
                                    if member.is_some(){
                                        (start_line..read_line+1).into_iter().for_each(|idx| 
                                            if idx < lines.len(){
                                                lines.remove(idx);
                                            }
                                        );
                                    }
                                    if end{
                                        // let end_line = &lines[read_line - 1];
                                        // let end_line1 = &lines[read_line - 2];
                                        // let end_line2 = &lines[read_line - 3];
                                        class_state = ClassBlockState::NotIn;
                                        // if member.is_some(){
                                        lines.insert(read_line,"};".to_string());
                                        engine.classes.push(std::mem::take(&mut class_info));
                                        // }
                                    }
                                    continue;
                                },
                                ClassFieldState::Public => {
                                    let (member, end) = parse_class_member(lines, &mut read_line, Some(&class_info.name));
                                    //keep menbers
                                    if let Some(member) = member{
                                        match member {
                                            ClassMember::Property(prop) => {
                                                class_info.properties.push(prop);
                                            },
                                            ClassMember::Function(cfn) => {
                                                if !cfn.name.is_empty(){
                                                    class_info.public_apis.push(cfn);
                                                }
                                            },
                                            ClassMember::Empty => (),
                                            ClassMember::Brace(_) => (),
                                        }
                                    }
                                    if end{
                                        // let end_line = &lines[read_line - 1];
                                        // let end_line1 = &lines[read_line - 2];
                                        // let end_line2 = &lines[read_line - 3];
                                        class_state = ClassBlockState::NotIn;
                                        // if member.is_some(){
                                        lines.insert(read_line,"};".to_string());
                                        engine.classes.push(std::mem::take(&mut class_info));
                                        // }
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
    #[allow(unused)]
    Empty,
}
// ///(int field)
// struct Brackets{
//     content: Option<String>,
// }
///分析类成员信息，并返回类是否结束
fn parse_class_member(lines: &mut Vec<String>, read_line: &mut usize, class_name: Option<&String>) -> (Option<ClassMember>, bool){
    if ignore_class_or_struct(lines, read_line){
        *read_line = read_line.saturating_sub(1);
        return (None, false);
    }
    if let Some(function) = parse_function(lines, read_line, class_name){
        return (Some(ClassMember::Function(function)), false);
    }
    else if let Some(property) = parse_field(lines, read_line) {
        return (Some(ClassMember::Property(property)), false);
    }
    else if let Some(_enum_type) = remove_enum(lines, read_line){
        return (Some(ClassMember::Property(Default::default())), false);
    }
    match parse_remove_brace(lines, read_line, true) {
        Some(brace) => {
            match brace {
                BraceType::Start => return (Some(ClassMember::Brace(brace)), false),
                BraceType::End => return (Some(ClassMember::Brace(brace)), true),
                BraceType::StartAndEnd(_) => {
                    return (Some(ClassMember::Brace(brace)), true);
                },
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
fn remove_fn_block(lines: &mut Vec<String>, read_line: &mut usize) -> Option<String>{
    if *read_line >= lines.len(){
        return None;
    }
    let line = lines[*read_line].trim();
    //find if first line contains { }
    let start = line.find("{");
    let end = line.rfind("}");
    let mut contents = vec![];
    match (start, end) {
        (Some(start), Some(end)) => {
            assert!(start < end, "invalid cuntion block");
            let content = line[start + 1..end].trim().to_string();
            lines[*read_line] = line[0..start].to_string() + ";";
            return Some(content);
        },
        (None, Some(_)) => panic!("unexpected line {}", line),
        (Some(start), None) => {
            let remain = line[start..].trim().to_string();
            if !remain.is_empty(){
                contents.push(remain);
            }
            lines[*read_line] = line[0..start].to_string() + ";";
        },
        (None, None) => {
            //add ;
            if line.ends_with(";"){
                return None;
            }
            lines[*read_line].push_str(";");
            *read_line += 1;
            while *read_line < lines.len() {
                let line_content = lines.remove(*read_line);
                if let Some(brace_start) = line_content.find("{"){
                    let block_end = line_content.contains("}");
                    let end = line_content.find("}").unwrap_or(line_content.len());
                    if brace_start < line_content.len(){
                        let content = line_content[brace_start + 1..end].trim().to_string();
                        if !content.is_empty(){
                            contents.push(content);
                        }
                    }
                    if block_end{
                        *read_line -= 1;
                        return Some(contents.join("\r\n"));
                    }
                    break;
                }
            }
        }
    }
    
    let mut brace = 0;
    while *read_line < lines.len() {
        let content = lines.remove(*read_line);
        if content.contains("{"){
            brace += 1;
        }
        if content.contains("}"){
            if brace == 0{
                if let Some(index) = content.find("}"){
                    let left = content[0..index].trim().to_string();
                    if !left.is_empty(){
                        contents.push(left);
                    }
                }
                break;
            }
            brace -= 1;
        }
        contents.push(content);
    }
    *read_line -= 1;
    if contents.is_empty(){
        None
    }
    else{
        Some(contents.join("\r\n"))
    }
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
#[allow(unused)]
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
fn parse_function(lines: &mut Vec<String>, read_line: &mut usize, class_name: Option<&String>) -> Option<CppApi>{    
    let mut line = lines[*read_line].trim().to_string();
    let start = line.find("(");
    let end = line.rfind(")");
    let equal = line.find("=");
    // let space = line.find(" ").or(line.find("\t"));
    match (start, end) {
        (Some(start), Some(end)) => {
            //generic, maybe property
            if let (Some(_bleft), Some(_bright)) = (line.find("<"), line.find(">")){
                // if start > bleft && start < bright {
                //     return None;
                // }
                // if end > bleft && end < bright{
                //     return None;
                // }
                //empty api
                return Some(CppApi::default());
            }
            //if () is type casting
            if let Some(eidx) = equal{
                if eidx < start{
                    return None;
                }
            }
            //function defination
            let mut override_flag = false;
            if let Some(_index) = line.find("override"){
                line = line.replace("override", "").trim().to_string();
                override_flag = true
            }
            // if let Some(ref_idx) = line.rfind("&"){

            // }
            let parameters = line[start + 1..end]
            .replace(" &", "&")
            .replace("< ", "<")
            .replace(" >", ">")
            .split(",")
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut param = s.to_string();
                if let Some(ridx) = param.rfind("&"){
                    //add space after & exp: void fn_ref(const Fptr&xptr, int v)
                    if ridx + 1 < param.len(){
                        let sr = &param[ridx + 1..=ridx + 1];
                        if sr != " " && sr != "\t"{
                            param.insert(ridx + 1, ' ');
                        }
                    }
                }
                let parameter = param.split(split_space).filter(|s| !s.is_empty()).map(|p| p.trim().to_string()).collect::<Vec<_>>();
                parameter
            })
            .collect::<Vec<_>>();
            let function_impl =  line[0..start]
            .replace(" *", "*")
            .replace(" &", "&")
            .replace("< ", "<")
            .replace(" >", ">")
            .split(split_space)
            .filter(|s| !s.is_empty())
            .map(|sa| sa.to_string())
            .collect::<Vec<_>>();
            let mut c_fn = CppApi::default();
            c_fn.is_override = override_flag;
            c_fn.class_name = class_name.cloned().unwrap_or_default();
            c_fn.parameters = super::parse_parameter(&parameters);
            super::parse_function(&mut c_fn, &function_impl);
            Some(c_fn)
        }
        _ => None,
    }
}
#[allow(unused)]
fn normalize_all(lines: &mut Vec<String>){
    let mut read_line = 0;
    while read_line < lines.len() {
        normalize_line(&mut lines[read_line]);
        read_line += 1;
    }
}
#[inline]
fn is_space(s: &str) -> bool{
    s == " " || s == "\t"
}
///格式化行
fn normalize_line(line: &mut String){
    if line.is_empty(){
        return;
    }
    let len = line.len();
    let mut index = line.len() - 1;
    let mut sharp_bracket = false;
    while index > 1 {
        let c = line[index..=index].chars().next().unwrap();
        match c{
            ' ' | '\t' => {
                if sharp_bracket{
                    //ignore <const Type>
                    if index > 5{
                        let const_expr = &line[..index];
                        if !const_expr.ends_with("const") &&
                        !const_expr.ends_with("const&"){
                            line.remove(index);
                        }
                    }
                    else{
                        line.remove(index);
                    }
                }
            },
            //space after ptr 
            '*' if !sharp_bracket && index + 1 < len => {
                if !is_space(&line[index+1..=index+1]){
                    line.insert(index + 1, ' ');
                }
            },
            //no space before >
            '>' => {
                sharp_bracket = true;
                if is_space(&line[index-1..index]){
                    index -= 1;
                    line.remove(index);
                }
            },
            //no space after <
            '<' if index + 1 < len => {
                if is_space(&line[index+1..=index+1]){
                    line.remove(index + 1);
                }
                sharp_bracket = false;
            }
            //space after &(&&)
            '&' if index + 1 < len => {
                //if next char is not &, then this is ref type
                if &line[index + 1..=index + 1] != "&"{
                    if !is_space(&line[index + 1..=index + 1]){
                        line.insert(index + 1, ' ');
                    }
                }
            }
            _ => ()
        }
        index -= 1;
    }
}
fn parse_field(lines: &mut Vec<String>, read_line: &mut usize) -> Option<CppProperty>{
    let mut line = lines[*read_line].trim().to_string();
    if line.contains(";") && !line.contains("}"){
        let mut property = CppProperty::default();
        if let (Some(_), Some(_)) = (line.find("["), line.find("]")){
            //todo array type do not export
            property.unsupported = true;
            return Some(property);
        }
        else if let (Some(_), Some(_)) = (line.find("<"), line.find(">")){
            //todo generic type do not export
            property.unsupported = true;
            return Some(property);
        }
        //type cast
        if let (Some(equal), Some(bstart), Some(bend)) = 
            (line.find("="), line.find("("), line.find(")")){
            //not type cast
            if equal > bstart{
                return None;
            }
            //remove type casting
            line.remove(bend);
            line = line.replace(&line[equal + 1..bstart + 1], " ");
        }
        if let Some(index) = line.find(":"){
            //add space
            let left = line[index - 1..index].to_string();
            let right = line[index + 1..line.len().min(index + 2)].to_string();
            if right != " " && right != "\t"{
                line.insert_str(index + 1, " ");
            }
            if left != " " && left != "\t"{
                line.insert_str(index - 1, " ");
            }
        }
        let property_info = line
        .replace("enum ", "")
        .replace("struct ", "")
        .replace("class ", "")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" *", "*")
        .replace(";", "")
        .replace(", ", "")
        .split("=")
        .next().unwrap()
        .split(split_space).filter(|s| !s.is_empty()).map(|s| s.to_string()).collect::<Vec<_>>();
        if property_info.len() < 2{
            return None;
        }
        if property_info.len() > 5{
            panic!("unsupported property defination {line}")
        }
        let count = property_info.len();
        let bit_flag = line.contains(":");
        let mut iter = property_info.into_iter().rev();
        if bit_flag{
            // name : value
            if count > 3{
                iter.next();
                iter.next();
                property.name = iter.next().expect(&format!("unknow property {}", line));
            }
            // name:value
            else{
                let name_slice = iter.next().expect(&format!("unknow property {}", line));
                property.name = name_slice.split(":").next().expect(&format!("unknow property {}", line)).to_string();
            }
        }
        else{
            property.name = iter.next().expect(&format!("unknow property {}", line));
        }
        property.type_str = iter.next().expect(&format!("unknow property {}", line));
        if let Some(index) = property.type_str.find("*"){
            property.type_str = property.type_str[0..index].to_string();
            property.is_ptr = true;
        }
        let (r_t, vt) = super::parse_c_type(&property.type_str);
        property.r_type = r_t;
        property.value_type = vt as i32;
        if let Some(flag) = iter.next(){
            match flag.trim(){
                "static" => {
                    property.is_static = true;
                },
                "const" => property.is_const = true,
                "extern" => (),
                "mutable" => (),
                "constexpr" => (),
                "volatile" => (),
                other => panic!("unsupported property defination {other} {line}")
            }
        }
        Some(property)
    }
    else{
        None
    }
}
///解析枚举
fn remove_enum(lines: &mut Vec<String>, read_line: &mut usize) -> Option<CppEnum>{
    if lines[*read_line].contains("enum"){
        let _line = lines.remove(*read_line);//.trim().replace("enum ", "").replace("class ", "").to_string();
        while !lines.remove(*read_line).contains("}") {
            //nop
        }
        Some(CppEnum::default())
    }
    else{
        None
    }
}
///忽略代码中的局部类和结构体(class 内部)
fn ignore_class_or_struct(lines: &mut Vec<String>, read_line: &mut usize) -> bool{
    if *read_line >= lines.len(){
        return false;
    }
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
                brace += 1;
            }
            else if line.ends_with("}") || line.ends_with("};"){
                if brace == 0{
                    break;
                }
                brace -= 1;
            }
        }
        lines.insert(*read_line, "".to_string());
        true
    }
    else{
        false
    }
}