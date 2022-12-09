
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FColor{
    pub B: u8,
    pub G: u8,
    pub R: u8,
    pub A: u8,
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FLinearColor{
	pub R: f32,
	pub G: f32,
	pub B: f32,
	pub A: f32
}
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FIntVector{
	pub X: i32,
	pub Y: i32,
	pub Z: i32
}
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FIntPoint{
	pub X: i32,
	pub Y: i32
}
pub fn char_str_2_string(cstr: *const std::os::raw::c_char) -> String{
    let c_str = unsafe {std::ffi::CStr::from_ptr(cstr)};
    match c_str.to_str(){
        Ok(s) => {
            let copied = s.to_string();
            unsafe{ FFIFreeStringInvokerHandler.as_ref().unwrap()(cstr) };
            return copied;
        },
        Err(e) => {
            error!("fail to parse host {:?}", e);
            unsafe{ FFIFreeStringInvokerHandler.as_ref().unwrap()(cstr) };
            return String::default();
        }
    }
}

/// rust string to c string(as parameter)
/// ```
/// pub fn string_2_char_str(rstr: String) -> *const std::os::raw::c_char{
///     string_2_char_strx!(rstr, rstr);/// 
///     rstr
/// }
/// ```
macro_rules! string_2_char_str{
    ($rstr: expr, $name:ident) => {
        let $name = CString::new($rstr).unwrap();
        let $name = $name.as_ptr();
    };
}

type FFIFreeStringInvoker = unsafe extern "C" fn(*const std::os::raw::c_char);
pub(super) static mut FFIFreeStringInvokerHandler: Option<FFIFreeStringInvoker> = None;
#[no_mangle]
extern "C" fn set_FFIFreeString_handler(handler: FFIFreeStringInvoker){
    unsafe{ FFIFreeStringInvokerHandler = Some(handler) };
}