# UnrealObject2RustBuilder
simple unreal object builder from c/cpp to rust

#usage:
    Edit configs/CustomSettings.json
    {
    "EngineRoot": "F:/UE5/UnrealEngine-5.0.0-early-access-2/Engine",
    "ExportClasses":[
        "UObject",
        "AActor",
        "ASpawn",
        "AController",
        "UActorComponent",
        "APlayerController",
        "FVector",
        "FVector4",
        "FVector2D",
        "FRotator",
        "FQuat",
        "FIntPoint",
        "FIntVector",
        "FColor",
        "FLinearColor",
        "FMatrix",
        "FTransform",
        "FRandomStream",
        "FPlane" 
    ],
    "ForceOpaque": [
        "UObject"
    ],
    "BlackList": [

    ],
    "ExportApis":[
        
    ],
    "ExportEnums":[

    ],
    "ExportConsts":[

    ],
    "ExportPathRoot": [
        "Runtime/AIModule/Public",
        "Runtime/BlueprintRuntime/Public",
        "Runtime/CinematicCamera/Public",
        "Runtime/Core/",
        "Runtime/UnrealAudio/Public",
        "Runtime/TimeManagement/Public",
        "Runtime/Renderer/Public",
        "Runtime/RenderCore/Public",
        "Runtime/PhysicsCore/Public",
        "Runtime/Navmesh/Public",
        "Runtime/InputCore/Public",
        "Runtime/Engine/",
        "Runtime/CoreUObject/Public",
        "Runtime/Experimental/",
        "Runtime/Landscape/Public",
        "Runtime/RuntimeAssetCache/Public",
        "Runtime/UnrealAudio/Public"
    ],
    "IgnoreFiles":[
        "Android*",
        "AnimNode_SaveCachedPose",
        "AIPerception*",
        "*Allocator",
        "Apple*",
        "AR*",
        "VR*",
        "XR*",
        "Linux*",
        "Archive*",
        "MacPlatform*",
        "Windows*",
        "WinHttp*",
        "AssetDataTagMapSerializationDetails",
        "Assertion*",
        "Ascii*",
        "Base64*",
        "IHead*",
        "GenericPlatform*",
        "IOS*",
        "*Test",
        "Test*",
        "AudioStreamingCache",
        "Blake3",
        "BlendProfile*",
        "Malloc*",
        "Build*",
        "D3D*",
        "OpenGl*",
        "Debug*",
        "Delegate*",
        "Demo*",
        "Developer*",
        "Devicce*",
        "EnvQueryTest*",
        "Generic*",
        "Http*",
        "Interface*",
        "Interp*",
        "Json*",
        "Mobile*",
        "Package*",
        "Patch*",
        "Platform*",
        "Xmpp*",
        "Xml*",
        "ChaosDebugDraw",
        "RuntimeAssetCacheEntryMetadata",
        "SlateDynamicImageBrush",
        "VideoConverter",
        "VideoConverterUVD",
        "TextFormatter",
        "ConstraintInstanceBlueprintLibrary",
        "LegacyScreenPercentageDriver",
        "BoundShaderStateCache",
        "RHI*",
        "StaticMeshAttributes",
        "NoExportTypes"
    ]
}
