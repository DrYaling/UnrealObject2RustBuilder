# UnrealObject2RustBuilder
simple unreal object builder from c/cpp to rust

# Json Object Struct
```
{
    "unreal_type": "FVector",  //unreal object name
    "type": "class",           //object type, struct, class, enum, or const(simple #define like #define CONSTANT 1) 
    "opaque_type": "false",    //if true, this object will just export a void* ptr, "false" or no "opaque_type" field will export listed apis and properties
    "alias": "",               //rust object name, same as unreal_type field if empty
    "properties": [ "X" ],     //properties will be exported
    "apis": ["GetMax"]         //apis will be exported

}
```
# notice that
    if a unreal object "type" field is enum or const, then other fields are not necessary

    if a unreal object "type" field is class, then the exported rust object will not impl "Copy" trait

    if a unreal object "apis" contains destructor(func like ~Name), then the exported rust object will not impl "Copy" trait