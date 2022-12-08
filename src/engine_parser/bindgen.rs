use std::{vec, collections::BTreeMap, sync::Mutex};

use crate::{is_primary, is_rs_primary};

use super::{
    unreal_engine::{
        Engine, 
        UnrealClass
    }, 
    config::{
        CustomSettings, 
        CppProperty, 
        CppApi
    }
};
static EXPORTED: Mutex<Vec<TypeImpl>> = Mutex::new(Vec::new());
#[derive(Clone, Debug, Default)]
struct TypeImpl{
    pub name: String,
    pub alis: String,
    #[allow(unused)]
    pub is_opaque: bool,
}
struct CodeGenerator{
    include: Vec<String>,
    header: Vec<String>,
    source: Vec<String>,
    registers: Vec<String>,
    api_defines: Vec<String>,
    rs_source: Vec<String>,
    rs_ffis: Vec<String>,
    type_impl: Vec<TypeImpl>,
    default_rs_header: usize,
}
impl CodeGenerator{    
    pub fn insert_rs_type(&mut self, type_str: &str, engine: &Engine, settings: &CustomSettings) -> TypeImpl{
        if type_str == "()"{
            return  TypeImpl{
                name: type_str.to_string(),
                alis: "c_void".to_string(),
                ..Default::default()
            };
        }
        if is_rs_primary(type_str){
            return  TypeImpl{
                name: type_str.to_string(),
                alis: type_str.to_string(),
                ..Default::default()
            };
        }
        return self.insert_type(type_str, engine, settings);
    }
    ///insert opaque type
    pub fn insert_type(&mut self, type_str: &str, engine: &Engine, settings: &CustomSettings) -> TypeImpl{
        //if type is not opaque and exported, use no alis
        if export_type(type_str, settings) && !is_opaque(type_str, engine, settings){
            return  TypeImpl{
                name: type_str.to_string(),
                alis: type_str.to_string(),
                ..Default::default()
            };
        }
        let type_str = if let Some(index) = type_str.find("Opaque"){
            &type_str[0..index]
        }
        else{
            type_str
        };
        if let Some(tp) = self.type_impl.iter().find(|tp| tp.name.as_str() == type_str) {
            return tp.clone();
        }
        let tp = TypeImpl{
            name: type_str.to_string(),
            alis: format!("{}Opaque", type_str),
            is_opaque: true,
        };
        self.type_impl.push(tp.clone());
        tp
    }
}
impl Default for CodeGenerator{
    fn default() -> Self {            
        let header: Vec<String> = vec![
            "#pragma once".into(),
            "".into(),
            "//extern \"C\"{".into(),

        ];
        let source: Vec<String> = vec![
            "#include \"CoreMinimal.h\"".into(),
            "#include \"Bindings.h\"".into(),
            "#include \"GameFramework/Actor.h\"".into(),
            "#include \"Containers/UnrealString.h\"".into(),
            "#include \"EngineUtils.h\"".into(),
            "#include \"Kismet/GameplayStatics.h\"".into(),
            "#include \"GameFramework/PlayerInput.h\"".into(),
            "#include \"Camera/CameraActor.h\"".into(),
            "#include \"Components/PrimitiveComponent.h\"".into(),
            "#include \"Sound/SoundBase.h\"".into(),
            "#include \"VisualLogger/VisualLogger.h\"".into(),
        ];
        
        let rs_source = vec![
            "#![allow(non_snake_case)]".to_string(),
            "#![allow(non_upper_case_globals)]".to_string(),
            "#![allow(non_camel_case_types)]".to_string(),
            "#![allow(dead_code)]".to_string(),
            "#![allow(unused_imports)]".to_string(),
            "use std::{ffi::c_void, os::raw::c_char, ops::{Deref, DerefMut}};".to_string(),
            "use ffis::*;".to_string(),
            r#"pub struct RefResult<T: Sized>{
    t: T,
}
impl<T: Sized> RefResult<T>{
    fn new(t: T) -> Self{
        Self { t }
    }
    pub fn get(&self)-> &T{
        &self.t
    }
    pub fn get_mut(&mut self)-> &mut T{
        &mut self.t
    }
}
impl<T: Sized> Deref for RefResult<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.t
    }
}
impl<T: Sized> DerefMut for RefResult<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.t
    }
}"#.to_string(),
        ];
        CodeGenerator{
            include: vec![
                "#include <cstdarg>".into(),
                "#include <cstdint>".into(),
                "#include <cstdlib>".into(),
                "#include <ostream>".into(),
            ],
            source,
            header,
            default_rs_header: rs_source.len(),
            rs_source,
            registers: vec![],
            api_defines: vec![
                "#include \"Binder.h\"".to_string(),
            ],
            type_impl: vec![],
            rs_ffis: vec![
                "\tmod ffis{".to_string(),
                "\tuse super::*;".to_string(),
                ],
        }
    }
}
///property can be export to ffi binder
pub fn should_export_property(engine: &Engine, property: &CppProperty) -> bool{
    !property.unsupported && 
    !property.is_generic &&
    (
        crate::is_primary(&property.type_str) ||
        engine.classes.iter().find(|class| class.name == property.type_str).is_some() ||
        engine.enums.iter().find(|eu| eu.name == property.type_str).is_some()
    )
}

///api can be export to ffi binder
pub fn should_export_api(engine: &Engine, api: &CppApi) -> bool{
    //api return type or parameters not generic
    !api.is_destructor &&
    !api.is_generic &&
    //all apis ref types are exported
    {
        let mut types = vec![api.rc_type.as_str()];
        api.parameters.iter().for_each(|param| types.push(param.type_str.as_str()));
        for tsr in &types {
            if !crate::is_primary(tsr){
                if engine.classes.iter().find(|class| class.name.as_str() == *tsr).is_none() &&
                    engine.enums.iter().find(|eu| eu.name.as_str() == *tsr).is_none(){
                    return false
                }
            }
        }
        true
    }
}
pub fn generate(engine: &Engine, settings: &CustomSettings) -> anyhow::Result<()>{
    let mut generator = CodeGenerator::default();
    for class in &settings.ExportClasses{
        if let Some(engine_class) = engine.classes.iter().find(|cls| &cls.name == class){
            gen_class(engine, engine_class, &mut generator, settings)?;
        }
    }
    generator.rs_ffis.push("}".to_string());
    //ffi apis
    generator.rs_source.append(&mut generator.rs_ffis);
    generator.header.push("//}".into());
    generator.header.insert(1, generator.include.join("\r\n"));
    let api_defines = generator.api_defines.join("\r\n");
    let api_registers = format!(r#"
#include "CoreMinimal.h"
#include "FFI.h"
#include "Binder.h"
class Plugin{{
    public:
    virtual void* GetDllExport(FString apiName) = 0;
}};
void register_all(Plugin* plugin){{
    {}
}}"#, generator.registers.join("\r\n"));
    generator.rs_source.insert(
        generator.default_rs_header, 
        generator.type_impl
        .iter()
        .map(|t| format!("pub type {} = c_void;//cpp type {}", t.alis, t.name))
        .collect::<Vec<_>>().join("\r\n")
    );
    std::fs::write("binders/cpp/Binder.h", generator.header.join("\r\n"))?;
    std::fs::write("binders/cpp/Binder.cpp", generator.source.join("\r\n"))?;
    std::fs::write("binders/cpp/FFI.h", api_defines)?;
    std::fs::write("binders/cpp/Registers.h", api_registers)?;
    std::fs::write("binders/rs/binders.rs", generator.rs_source.join("\r\n"))?;
    Ok(())
}
fn is_opaque(type_str: &str, engine: &Engine, settings: &CustomSettings) -> bool{
    if settings.ForceOpaque.iter().find(|fo| fo.as_str() == type_str).is_some(){
        return true;
    }
    if is_primary(type_str){
        return  false;
    }
    if let Some(class) = engine.classes.iter().find(|tp| {
        tp.name.as_str() == type_str        
    }){
        !(class.none_public_properties.is_empty() && 
        class.properties.iter().find(|p| {
            p.bit_value || !should_export_property(engine, p)
        }).is_none())
    }
    else{
        true
    }
}
fn gen_class(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, settings: &CustomSettings) -> anyhow::Result<()>{
    if let Some(old) = EXPORTED.lock().unwrap().iter().find(|c| c.name == class.name){
        println!("try to export class {} at path {}, but class already exported at path {}", class.name, class.path, old.alis);
        return Ok(());
    }
    //是否不透明对象
    //需要满足4点条件
    //1.所有成员变量是public
    //2.所有成员变量不是位变量
    //3.所有成员变量不是泛型
    //4.所有成员变量类型均在导出列表中(或者基本数据类型(非字符串))
    let is_opaque = is_opaque(&class.name, engine, settings);
    EXPORTED.lock().unwrap().push(TypeImpl{name: class.name.clone(), alis: class.path.clone(), is_opaque});
    //header
    let include = format!("#include \"{}\"", class.path);
    if !generator.include.contains(&include){
        generator.include.push(include);
    }
    if is_opaque{
        gen_opaque(engine, class, generator, settings)?;
    }
    else{
        gen_none_opaque(engine, class, generator, settings)?
    }
    generator.rs_source.push("}".to_string());
    Ok(())
}
///生成不透明对象的绑定信息
fn gen_opaque(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, settings: &CustomSettings) -> anyhow::Result<()>{
    let ts = generator.insert_type(&class.name, engine, settings);
    //type imply
    // generator.header.push(format!("\tusing {} = void;", object_name));
    //rust type impl
    // generator.rs_source.push(format!("pub type {} = c_void;", object_name));
    generator.rs_source.push(format!(r#"pub struct {}{{
    inner: *mut {}
}}
impl {}{{
    #[inline]
    pub fn inner(&self) -> *mut {} {{ self.inner }}"#, class.name, ts.alis, class.name, ts.alis));
    parse_properties(engine, class, generator, true, settings)?;
    parse_functions(engine, class, generator, true, settings)?;
    Ok(())
}
///生成透明对象的绑定信息
fn gen_none_opaque(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, settings: &CustomSettings) -> anyhow::Result<()>{
    let object_name = format!("{}", class.name);
    //rust type impl
    generator.rs_source.push(format!(r#"#[repr(C)]
pub struct {}{{
{}
}}
impl {}{{"#, 
    object_name, 
    class.properties.iter().map(|field|{
        format!("\tpub {}: {}", field.name, field.r_type)
    }).collect::<Vec<_>>().join(",\r\n"),
    object_name));
    // parse_properties(engine, class, generator, &mut rs_handlers)?;
    parse_functions(engine, class, generator, false, settings)?;
    Ok(())
}
fn export_type(type_str: &str, settings: &CustomSettings) -> bool{
    is_primary(type_str) || settings.ExportClasses.iter().find(|x| x.as_str() == type_str).is_some()
}
///生成函数
fn parse_functions(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, opaque: bool, settings: &CustomSettings) -> anyhow::Result<()>{    
    let class_name = class.name.clone();
    // let cpp_class_atlas = "void".to_string();
    // let rs_class_atlas = if opaque {class_name.clone() + "Opaque"}else{class_name.clone()};
    //api map key is api name, value is api with overload count
    let mut api_map: BTreeMap<String, i32> = BTreeMap::default();
    'api: for api in &class.public_apis {        
        let parameters = api.parameters.clone();
        if api.name == "AdditionalStatObject"{
            print!("pause");
        }
        //api with opaque(and not exported) none ptr parameter  will not export
        for param in &api.parameters {
            if is_opaque(&param.type_str, engine, settings) && !param.ptr_param{
                continue 'api;
            }
            if !param.ptr_param && !export_type(&param.type_str, settings){
                continue 'api;
            }
        }
        //generic do not export
        if !should_export_api(engine, api){
            continue;
        }
        let opaque_ret = is_opaque(&api.rc_type, engine, settings);
        //opaque is ptr or ref, else not export
        if opaque_ret && !api.ptr_ret&& !api.ref_ret{
            continue;
        }
        //opaque but didn't export
        else if !opaque_ret && !export_type(&api.rc_type, settings){
            continue;
        }
        let mut designed_api_name = api.name.clone();
        if let Some(v) = api_map.get_mut(&designed_api_name){
            *v += 1;
            designed_api_name.push_str(&v.to_string());
        }
        else{
            api_map.insert(designed_api_name.clone(), 1);
        }
        //function result with liftime
        let mut lifetime_ret = false;
        //make ref result to ptr between ffi api
        let mut ref_to_ptr = false;
        let rs_ret_type = generator.insert_rs_type(&api.r_type, engine, settings);
        let (cpp_ret, rs_ret_liftime, rs_ret_origin) = match opaque_ret {
            true => {
                //ref
                if api.ref_ret{
                    lifetime_ret = true;
                    ref_to_ptr = true;
                    // ("void*".to_string(), format!(" -> RefResult<*mut {}>", rs_ret_type.alis), format!(" -> *mut {}", rs_ret_type.alis))
                    ("void*".to_string(), format!(" -> &'a {}", rs_ret_type.alis), format!(" -> *mut {}", rs_ret_type.alis))
                }
                //ptr 
                else{
                    ("void*".to_string(), format!(" -> *mut {}", rs_ret_type.alis), format!(" -> *mut {}", rs_ret_type.alis))
                }
            },
            false => {
                if api.rc_type == "void"{
                    (api.rc_type.to_string(), format!(""), format!(""))
                }
                else{
                    if api.ptr_ret{
                        (format!("{}*", api.rc_type), format!(" -> *mut {}", rs_ret_type.alis), format!(" -> *mut {}", rs_ret_type.alis))
                    }
                    else if api.ref_ret{
                        lifetime_ret = true;                        
                        ref_to_ptr = true;
                        //ref in ffi is not supported, so translate to ptr
                        (format!("{}*", api.rc_type), format!(" ->&'a {}", rs_ret_type.name), format!(" -> *mut {}", rs_ret_type.name))
                    }
                    else{
                        (api.rc_type.to_string(), format!(" -> {}", rs_ret_type.name), format!(" -> {}", rs_ret_type.name))
                    }
                }
            }
        };
        let mut rs_parameters = vec![];
        let mut rs_ffi_parameters = vec![];
        let mut rs_fn_parameters = vec![];
        let pstr = parameters.into_iter().map(|prop|{
            //ffi value should be transform as ptr or value
            let rs_tag;
            let tag = if prop.ptr_param{
                rs_tag = "*mut ";
                "*"
            }
            else if prop.ref_param{
                rs_tag = "&";
                "&"
            }
            else{
                rs_tag = "";
                ""
            };
            let ts = generator.insert_rs_type(&prop.r_type, engine, settings);
            rs_ffi_parameters.push(format!("{}{}", rs_tag, ts.alis));
            rs_parameters.push(format!("{}", prop.name));
            rs_fn_parameters.push(format!("{}: {}{}", prop.name,  rs_tag, ts.alis));
            format!("{}{tag} {}", prop.type_str, prop.name)
        }).collect::<Vec<_>>();
        let mut full_proper = pstr.clone();
        if !api.is_construstor && !api.is_static{
            full_proper.insert(0, format!("void* target"));
            rs_ffi_parameters.insert(0, format!("*mut c_void"));
            //rust "this"
            if opaque{
                rs_parameters.insert(0, format!("self.inner"));
            }
            else{
                if api.is_const{
                    rs_parameters.insert(0, format!("self as *const Self as *mut c_void"));
                }
                else{
                    rs_parameters.insert(0, format!("self as *mut Self as *mut c_void"));
                }
            }
            if api.is_const{
                rs_fn_parameters.insert(0, format!("&self"));
            }
            else{
                rs_fn_parameters.insert(0, format!("&mut self"));
            }
        }
        let parameter_name_list = api.parameters
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
        //constructor
        let cpp_api_name = if api.is_construstor{
            let api_name = format!("uapi_{}_New{}", class_name, api.parameters.len());
            let content = format!(r#"   {} {}({}){{
        return new {}({});
    }}"#, cpp_ret, api_name, full_proper.join(", "),
            class_name, parameter_name_list);
            generator.header.push(content);
            api_name
        }
        //others
        else{
            //cpp ret result caster
            let result_caster = if ref_to_ptr {
                format!("({})&", cpp_ret)
            }else{
                if api.ptr_ret{
                    format!("({})", cpp_ret)
                }
                else{
                    "".to_string()
                }
            };
            let api_name = format!("uapi_{}_{}", class_name, designed_api_name);
            let return_flag = if cpp_ret == "void"{""}else{"return "};
            let content = if api.is_static{
                format!(r#"   {} {}({}){{
        {} {}{}::{}({});
    }}"#, cpp_ret, api_name, full_proper.join(", "), return_flag,
    result_caster, class_name, api.name, parameter_name_list)
            }
            else{
                format!(r#"   {} {}({}){{
        {} {}(({}*)target)->{}({});
    }}"#, cpp_ret, api_name, full_proper.join(", "), return_flag,
    result_caster, class_name, api.name, parameter_name_list)
            };
            generator.header.push(content);
            api_name
        };
        //rust ffi api
        //api with liftime
        let lifetime_tag = if lifetime_ret{"<'a>"}else{""}; 
        //if result is ref, add & tag before function call
        let mut ref_flag_tail = "";
        let ref_flag = if lifetime_ret {
            //opaque is ptr
            if opaque_ret{
                ref_flag_tail  = "";
                // "RefResult::new("
                "&*"
            }
            //else is ref
            else{
                "&*"
            }
        }else{""}; 
        //rust getter ffi api
        let callback_name = format!("{}_{}Invoker", class_name, designed_api_name);
        let callback_handler = format!("{callback_name}Handler");
        let ffi_api_name = format!("set_{}_{}_handler", class_name, designed_api_name);        
        let handler_code = format!(r#"
    type {callback_name} = unsafe extern "C" fn({}){};
    pub(super) static mut {callback_handler}: Option<{callback_name}> = None;
    #[no_mangle]
    extern "C" fn {ffi_api_name}(handler: {callback_name}){{
        unsafe{{ {callback_handler} = Some(handler) }};
    }}"#, rs_ffi_parameters.join(", "), rs_ret_origin);
        //get handler
        generator.rs_ffis.push(handler_code);

        // cpp register         
        generator.api_defines.push(format!(r#"
using {cpp_api_name}Fn = void(*)({}(*)({}));"#, cpp_ret, full_proper.join(",")));

        generator.registers.push(format!(r#"
    auto const api{cpp_api_name} = ({cpp_api_name}Fn)plugin->GetDllExport(TEXT("{ffi_api_name}\0"));
    if(api{cpp_api_name}){{
        api{cpp_api_name}(&{cpp_api_name});
    }}"#));  
        //rust member function
        let rs_block = format!(r#"
    #[inline]
    pub fn {}{}({}){}{{
        unsafe{{ {}{callback_handler}.as_ref().unwrap()({}){} }}
    }}"#,
        designed_api_name, lifetime_tag, 
        rs_fn_parameters.join(", "), rs_ret_liftime, 
        ref_flag, rs_parameters.join(", "), ref_flag_tail);
        generator.rs_source.push(rs_block);
    }
    Ok(())
}

///generate properties
fn parse_properties(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, opaque: bool, settings: &CustomSettings) -> anyhow::Result<()>{
    
    let class_name = class.name.clone();
    let cpp_class_atlas = "void";
    let rs_class_alas = if opaque {class.name.clone() + "Opaque"} else{ class.name.clone()};
    for property in &class.properties {
        if property.is_const || property.is_static || !should_export_property(engine, property){
            continue;
        }
        //opaque type will not export
        if is_opaque(&property.type_str, engine, settings){
            continue;
        }
        //cpp getter api
        let get_cpp_name = format!("get_{class_name}_{}", property.name);
        let content = format!(r#"
    {} {get_cpp_name}({cpp_class_atlas}* target) {{ return (({class_name}*)target) -> {};}};"#,
    property.type_str, property.name);
    
        generator.header.push(content);
        //
        //cpp setter pai
        let set_cpp_name = format!("set_{class_name}_{}", property.name);
        let content = format!(r#"
    void {set_cpp_name}({cpp_class_atlas}* target, {} value){{ (({class_name}*)target) -> {} = value;}};"#,
    property.type_str, property.name);
    
        generator.header.push(content);

        //rust code

        //pub type IterateActorsFn = unsafe extern "C" fn(array: *mut *mut AActorOpaque, len: *mut u64);
        //rust getter ffi api
        let callback_name = format!("Get{}Invoker", property.name);
        let callback_handler_get = format!("{callback_name}Handler");
        let api_name = format!("set_{}{}_get_handler", class_name, property.name);
        let handler_code = format!(r#"
    type {callback_name} = unsafe extern "C" fn(target: *mut {rs_class_alas}) -> {};
    pub(super) static mut {callback_handler_get}: Option<{callback_name}> = None;
    #[no_mangle]
    extern "C" fn {api_name}(handler: {callback_name}){{
        unsafe{{ {callback_handler_get} = Some(handler) }};
    }}"#, property.r_type);
        //get handler
        generator.rs_ffis.push(handler_code);
        //get cpp register
        //using EntryUnrealBindingsFn = uint32_t(*)(UnrealBindings bindings, RustBindings *rust_bindings);
        generator.api_defines.push(format!(r#"
using {api_name}Fn = void(*)({}(*)({cpp_class_atlas}* target));"#, property.type_str));

    generator.registers.push(format!(r#"
    auto const api{api_name} = ({api_name}Fn)plugin->GetDllExport(TEXT("{api_name}\0"));
    if(api{api_name}){{
        api{api_name}(&{get_cpp_name});
    }}"#));

        //rust setter ffi block
        let callback_name = format!("Set{}Invoker", property.name);
        let callback_handler_set = format!("{callback_name}Handler");
        let api_name = format!("set_{}{}_set_handler", class_name, property.name);
        let handler_code = format!(r#"
    type {callback_name} = unsafe extern "C" fn(target: *mut {rs_class_alas}, value: {});
    pub(super) static mut {callback_handler_set}: Option<{callback_name}> = None;
    #[no_mangle]
    extern "C" fn {api_name}(handler: {callback_name}){{
        unsafe {{{callback_handler_set} = Some(handler) }};
    }}"#, property.r_type);
        //setter
        generator.rs_ffis.push(handler_code);
        //cpp setter ffi api
        generator.api_defines.push(format!(r#"
using {api_name}Fn = void(*)(void(*)({cpp_class_atlas}* target, {} value));"#, property.type_str));
        generator.registers.push(format!(r#"    auto const api{api_name} = ({api_name}Fn)plugin->GetDllExport(TEXT("{api_name}\0"));
    if(api{api_name}){{
        api{api_name}(&{set_cpp_name});
    }}"#));

        //rust getter/setter source code
        generator.rs_source.push(format!(r#"
    #[inline]
    pub fn get_{}(&self) -> {}{{
        unsafe{{ {callback_handler_get}.as_ref().unwrap()(self.inner) }}
    }}"#, property.name, property.r_type));
        generator.rs_source.push(format!(r#"
    #[inline]
    pub fn set_{}(&mut self, value: {}){{
        unsafe{{ {callback_handler_set}.as_ref().unwrap()(self.inner, value) }}
    }}"#, property.name, property.r_type));
    }
    Ok(())
}