#pragma once
#include "RustApi.h"
void register_all(Plugin* plugin);
#define RSTR_TO_TCHAR(str, len) (TCHAR*)FUTF8ToTCHAR((const ANSICHAR*)str,(int32)len).Get()