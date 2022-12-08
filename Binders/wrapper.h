
//#include "CoreTypes.h"
//#include "Core/Public/Math/Color.h"

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