# UnrealObject2RustBuilder
simple unreal object builder from c/cpp to rust

#usage:

    Edit configs/CustomSettings.json
    
    set EngineRoot to ue5 soruce root
    
    run  project and see outputs in Binders/cpp/Binder.* and Binders/rs/binder.rs
 #dependency
 
    llvm(15.*) and clang.exe be set to system env "Path"
#known unsupported features

    PLATFORM_LITTLE_ENDIAN defined structures(almost all other macro defined content and may be wrong or export fail)
    
    generic types/apis and fields
    
    unions
    
