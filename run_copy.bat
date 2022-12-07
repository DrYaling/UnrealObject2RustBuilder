cargo run --release
if %errorlevel% == 1 goto exitFlag
copy Binders\rs\binders.rs src\binders.rs
:exitFlag