
#include "CoreMinimal.h"
#include "FFI.h"
#include "Binder.h"
class Plugin{
    public:
    void* GetDllExport(FString apiName);
}
void register_all(Plugin* plugin){
    
}