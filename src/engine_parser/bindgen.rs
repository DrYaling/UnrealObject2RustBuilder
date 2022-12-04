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
struct CodeGenerator{
    header: Vec<String>,
    source: Vec<String>,
    registers: Vec<String>,
    api_defines: Vec<String>,
    rs_source: Vec<String>,
}
impl Default for CodeGenerator{
    fn default() -> Self {            
        let header: Vec<String> = vec![
            "#pragma once".into(),
            "#include <cstdarg>".into(),
            "#include <cstdint>".into(),
            "#include <cstdlib>".into(),
            "#include <ostream>".into(),
            "#include <new>".into(),
            "".into(),
            "extern \"C\"{".into(),

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
        CodeGenerator{
            source,
            header,
            rs_source: vec![
                "use std::{ffi::c_void, os::raw::c_char};".to_string(),
            ],
            registers: vec![],
            api_defines: vec![],
        }
    }
}
///property can be export to ffi binder
pub fn should_export_property(engine: &Engine, property: &CppProperty) -> bool{
    !property.unsupported && 
    !is_generic(&property.type_str) &&
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
    (
        !is_generic(&api.rc_type) &&
        api.parameters.iter().find(|param| is_generic(&param.type_str)).is_none()
     ) &&
    //all apis ref types are exported
    {
        let mut types = vec![api.rc_type.as_str()];
        api.parameters.iter().for_each(|param| types.push(param.type_str.as_str()));
        if types.iter().find(|tsr| !crate::is_primary(**tsr)).is_some(){
            for tsr in &types {
                if engine.classes.iter().find(|class| class.name.as_str() == *tsr).is_none() &&
                    engine.enums.iter().find(|eu| eu.name.as_str() == *tsr).is_none(){
                    return false
                }
            }            
            true
        }
        else{
            true
        }
    }
}
pub fn generate(engine: &Engine, settings: &CustomSettings) -> anyhow::Result<()>{
    let mut generator = CodeGenerator::default();
    for class in &settings.ExportClasses{
        if let Some(engine_class) = engine.classes.iter().find(|cls| &cls.name == class){
            gen_class(engine, engine_class, &mut generator)?;
        }
    }
    generator.header.push("}".into());
    let api_defines = generator.api_defines.join("\r\n");
    let api_registers = format!(r#"
#include "CoreMinimal.h"
#include "FFI.h"
#include "Binder.h"
class Plugin{{
    public:
    void* GetDllExport(FString apiName);
}}
void register_all(Plugin* plugin){{
    {}
}}"#, generator.registers.join("\r\n"));
    std::fs::write("binders/cpp/Binder.h", generator.header.join("\r\n"))?;
    std::fs::write("binders/cpp/Binder.cpp", generator.source.join("\r\n"))?;
    std::fs::write("binders/cpp/FFI.h", api_defines)?;
    std::fs::write("binders/cpp/Registers.h", api_registers)?;
    std::fs::write("binders/rs/binders.rs", generator.rs_source.join("\r\n"))?;
    Ok(())
}
fn gen_class(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator) -> anyhow::Result<()>{
    generator.header.push(format!("\tusing {}Opaque = void;", class.name));
    generator.header.insert(7, format!("#include \"{}\"", class.path));
    generator.rs_source.push(format!("pub type {}Opaque = c_void;", class.name));
    generator.rs_source.push(format!(r#"pub struct {}{{
    inner: {}Opaque
}}
impl {}{{"#, class.name, class.name, class.name));
    let mut rs_handlers: Vec<String> = vec![];
    parse_properties(engine, class, generator, &mut rs_handlers)?;
    parse_functions(engine, class, generator, &mut rs_handlers)?;
    generator.rs_source.push("}".to_string());
    generator.rs_source.append(&mut rs_handlers);
    Ok(())
}
fn parse_functions(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, rs_handlers: &mut Vec<String>) -> anyhow::Result<()>{    
    let class_name = class.name.clone();
    let class_atlas = class_name.clone() + "Opaque";
    for api in &class.public_apis {        
        let parameters = api.parameters.clone();
        //generic do not export
        if !should_export_api(engine, api){
            continue;
        }
        let pstr = parameters.into_iter().map(|prop|{
            //ffi value should be transform as ptr or value
            let tag = if prop.ptr_param{
                "*"
            }
            else if prop.ref_param{
                "&"
            }
            else{
                ""
            };
            format!("{}{tag} {}", prop.type_str, prop.name)
        }).collect::<Vec<_>>();
        let mut full_proper = pstr.clone();
        if !api.is_construstor && !api.is_static{
            full_proper.insert(0, format!("{}* target", class_atlas));
        }
        let return_type = 
        if api.is_construstor{
            format!("{}*", class_atlas)
        }
        else if api.ptr_ret{
            format!("{}*", api.rc_type)
        }
        else{
            let r = api.rc_type.clone();
            if r.is_empty(){
                "void".into()
            }
            else{
                r
            }
        };
        let parameter_name_list = api.parameters
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
        //constructor
        if api.is_construstor{
            let content = format!(r#"   {} uapi_{}_New{}({}){{
                return new {}({});
            }}"#, return_type, class_name, api.parameters.len(), full_proper.join(", "),
            class_name, parameter_name_list);
            generator.header.push(content);
        }
        //others
        else{
            let return_flag = if return_type == "void"{""}else{"return "};
            let content = if api.is_static{
                format!(r#"   {} uapi_{}_{}({}){{
                    {} (({}*)target)->{}({});
                }}"#, return_type, class_name, api.name, full_proper.join(", "), return_flag,
                class_name, api.name, parameter_name_list)
            }
            else{
                format!(r#"   {} uapi_{}_{}({}){{
                    {} (({}*)target)->{}({});
                }}"#, return_type, class_name, api.name, full_proper.join(", "), return_flag,
                class_name, api.name, parameter_name_list)
            };
            generator.header.push(content);
        }
    }
    Ok(())
}
fn is_generic(ts: &str) -> bool{
    ts.contains("<")
}
///generate properties
fn parse_properties(engine: &Engine, class: &UnrealClass, generator: &mut CodeGenerator, rs_handlers: &mut Vec<String>) -> anyhow::Result<()>{
    
    let class_name = class.name.clone();
    let class_atlas = class_name.clone() + "Opaque";

    for property in &class.properties {
        if property.is_const || property.is_static || !should_export_property(engine, property){
            continue;
        }
        //cpp getter api
        let get_cpp_name = format!("get_{class_name}_{}", property.name);
        let content = format!(r#"
    {} {get_cpp_name}(const {class_atlas}* target) const{{ return (({class_name}*)target) -> {};}};"#,
    property.type_str, property.name);
    
        generator.header.push(content);
        //
        //cpp setter pai
        let set_cpp_name = format!("set_{class_name}_{}", property.name);
        let content = format!(r#"
    void {set_cpp_name}({class_atlas}* target, {} value){{ (({class_name}*)target) -> {} = value;}};"#,
    property.type_str, property.name);
    
        generator.header.push(content);

        //rust code

        //pub type IterateActorsFn = unsafe extern "C" fn(array: *mut *mut AActorOpaque, len: *mut u64);
        //rust getter ffi api
        let callback_name = format!("Get{}Invoker", property.name);
        let callback_handler_get = format!("{callback_name}Handler");
        let api_name = format!("set_{}{}_get_handler", class_name, property.name);
        let handler_code = format!(r#"
pub type {callback_name} = unsafe extern "C" fn(target: *mut {class_atlas}) -> {};
static mut {callback_handler_get}: Option<{callback_name}> = None;
#[no_mangle]
extern "C" fn {api_name}(handler: {callback_name}){{
    unsafe{{
        {callback_handler_get} = Some(handler);
    }}
}}
"#, property.r_type);
        //get handler
        rs_handlers.push(handler_code);
        //get cpp register
        //using EntryUnrealBindingsFn = uint32_t(*)(UnrealBindings bindings, RustBindings *rust_bindings);
        generator.api_defines.push(format!(r#"
using {api_name}Fn = {}(*)(const {class_atlas}* target);"#, property.type_str));

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
pub type {callback_name} = unsafe extern "C" fn(target: *mut {class_atlas}, value: {});
static mut {callback_handler_set}: Option<{callback_name}> = None;
#[no_mangle]
extern "C" fn {api_name}(handler: {callback_name}){{
    unsafe{{
        {callback_handler_set} = Some(handler);
    }}
}}
"#, property.r_type);
        //setter
        rs_handlers.push(handler_code);
        //cpp setter ffi api
        generator.api_defines.push(format!(r#"
using {api_name}Fn = void(*)(const {class_atlas}* target, {} value);"#, property.type_str));
        generator.registers.push(format!(r#"    auto const api{api_name} = ({api_name}Fn)plugin->GetDllExport(TEXT("{api_name}\0"));
    if(api{api_name}){{
        api{api_name}(&{set_cpp_name});
    }}"#));

        //rust getter/setter source code
        generator.rs_source.push(format!(r#"
        #[inline]
        pub fn get_{}(&self) -> {}{{
            {callback_handler_get}.as_ref().unwrap()(self.inner)
        }}"#, property.name, property.r_type));
        generator.rs_source.push(format!(r#"
        #[inline]
        pub fn set_{}(&mut self, value: {}){{
            {callback_handler_set}.as_ref().unwrap()(self.inner, value)
        }}"#, property.name, property.r_type));
    }
    Ok(())
}