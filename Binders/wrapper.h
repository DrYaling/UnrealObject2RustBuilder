
struct UColor{
    uint8 B,G,R,A;
};
//fixed translate name
UColor FColor2UColor(const FColor& input) {
    UColor ret;
    ret.B = input.B;
    ret.G = input.G;
    ret.R = input.R;
    ret.A = input.A;
    return ret;
}
//fixed translate name
FColor UColor2FColor(const UColor& input) {
    FColor ret;
    ret.B = input.B;
    ret.G = input.G;
    ret.R = input.R;
    ret.A = input.A;
    return ret;
}

struct ULinearColor{
    float R, B, G, A;
};
//fixed translate name
ULinearColor FLinearColor2ULinearColor(const FLinearColor& input) {
    ULinearColor ret;
    ret.B = input.B;
    ret.G = input.G;
    ret.R = input.R;
    ret.A = input.A;
    return ret;
}
//fixed translate name
FLinearColor ULinearColor2FLinearColor(const ULinearColor& input) {
    FLinearColor ret;
    ret.B = input.B;
    ret.G = input.G;
    ret.R = input.R;
    ret.A = input.A;
    return ret;
}

struct UIntVector{
    int32 X, Y, Z;
};
//fixed translate name
UIntVector FIntVector2UIntVector(const FIntVector& input) {
    UIntVector ret;
    ret.X = input.X;
    ret.Y = input.Y;
    ret.Z = input.Z;
    return ret;
}
//fixed translate name
FIntVector UIntVector2FIntVector(const UIntVector& input) {
    FIntVector ret;
    ret.X = input.X;
    ret.Y = input.Y;
    ret.Z = input.Z;
    return ret;
}
struct UIntPoint{
    int32 X, Y, Z;
};
//fixed translate name
UIntPoint FIntPoint2UIntPoint(const FIntPoint& input) {
    UIntPoint ret;
    ret.X = input.X;
    ret.Y = input.Y;
    return ret;
}
//fixed translate name
FIntPoint UIntPoint2FIntPoint(const UIntPoint& input) {
    FIntPoint ret;
    ret.X = input.X;
    ret.Y = input.Y;
    return ret;
}
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
using reset_rust_string_handler = void (*)(RefString utfstr, const char* c_str, uint32 size);
reset_rust_string_handler reset_rust_string = nullptr;
//thread unsafe
void ResetFStringBuffer(const FString& fstr, RefString& utfstr) {
    if (reset_rust_string) {
        auto pSendData = fstr.GetCharArray().GetData();
        char* dst = (char*)TCHAR_TO_UTF8(pSendData);
        if (dst) {
            const uint32 dataSize = strlen(dst);
            reset_rust_string(utfstr, dst, dataSize);
        }
        else {
            reset_rust_string(utfstr, "", 0);
        }
    }
}
void ResetFTextBuffer(const FText& fstr, RefString& utfstr) {
    ResetFStringBuffer(fstr.ToString(), utfstr);
}
using create_native_string_handler = char* (*)(const char* c_str, uint32);
create_native_string_handler create_native_string = nullptr;

FString Utf82FString(const NativeString& utfstr) {
    if (utfstr.utfStr && utfstr.size > 0)
        return FString(utfstr.size, RSTR_TO_TCHAR(utfstr.utfStr, utfstr.size));
    else
        return FString();
}
FString Utf8Ref2FString(const RefString& utfstr) {
    if (utfstr.utfStr && utfstr.size > 0)
        return FString(utfstr.size, RSTR_TO_TCHAR(utfstr.utfStr, utfstr.size));
    else
        return FString();
}
const char* FString2Utf8(FString fstr) {
    if (create_native_string == nullptr) {
        return nullptr;
    }
    TCHAR* pSendData = fstr.GetCharArray().GetData();
    const char* dst = (const char*)TCHAR_TO_UTF8(pSendData);
    auto const dataSize = strlen(dst);
    char* buffer = create_native_string(dst, dataSize);
    // UE_LOG(LogTemp, Display, TEXT("FString2Utf8 %s,ptr %p, size %d"), *fstr, buffer, dataSize);
    return buffer;
}
FName Utf82FName(const NativeString& utfstr) {
    auto fstr = Utf82FString(utfstr);
    return FName(*fstr);
}
const char* FName2Utf8(FName fname) {
    auto fstr = fname.ToString();
    return FString2Utf8(fstr);
}
FText Utf82FText(const NativeString& utfstr) {
    auto fstr = Utf82FString(utfstr);
    return FText::FromString(fstr);
}
FText Utf8Ref2FText(const RefString& utfstr) {
    auto fstr = Utf8Ref2FString(utfstr);
    return FText::FromString(fstr);
}
const char* FText2Utf8(FText text) {
    auto fstr = text.ToString();
    return FString2Utf8(fstr);
}