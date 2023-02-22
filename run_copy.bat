cargo run --release
if %errorlevel% == 1 goto exitFlag
copy Binders\rs\binders.rs ..\..\Unreal\WorkSpace\gameplay\engine-api\src\binders.rs
copy Binders\rs\enums.rs ..\..\Unreal\WorkSpace\gameplay\engine-api\src\enums.rs
copy Binders\cpp\Binder.cpp ..\..\Unreal\WorkSpace\gameplay\unreal\Source\RustGamePlay\Binder.cpp
:exitFlag