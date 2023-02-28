#pragma once
#include "RustApi.h"

//thread unsafe
struct NativeString {
    char* utfStr;
    uint32 size;
};
//thread unsafe
struct RefString {
    char* utfStr;
    void* str_ref;
    uint32 size;
};
FString Utf82FString(const NativeString& utfstr);
FString Utf8Ref2FString(const RefString& utfstr);
FName Utf82FName(const NativeString& utfstr);
FText Utf82FText(const NativeString& utfstr);
FText Utf8Ref2FText(const RefString& utfstr);
void register_all(Plugin* plugin);
#define RSTR_TO_TCHAR(str, len) (TCHAR*)FUTF8ToTCHAR((const ANSICHAR*)str,(int32)len).Get()
