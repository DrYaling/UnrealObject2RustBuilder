
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
    if cstr.is_null(){
        error!("translate string by null ptr");
        return Default::default();
    }
    unsafe{CString::from_raw(cstr as *mut _)}
    .to_str()
    .map_err(|e| {
        error!("fail to load string {:?}", e)
    })
    .map(|s| s.to_string())
    .unwrap_or_default()
}
#[no_mangle]
unsafe extern fn create_native_string(ptr: *const c_char, size: u32) -> *mut c_char {
    // Take the ownership back to rust and drop the owner
    let slice = std::slice::from_raw_parts(ptr, size as usize);
    let Ok(native_str) = String::from_utf8(slice.iter().map(|c| *c as u8).collect())
    else{
        error!("fail to parse from {:p}", ptr);
        return std::ptr::null_mut();
    };
    let Ok(native_str) = CString::new(native_str)
    else{
        error!("fail to parse from {:p}", ptr);
        return std::ptr::null_mut();
    };
    native_str.into_raw()
}
/// rust string to c string(as parameter)
/// ```
/// pub fn string_2_char_str(rstr: String) -> *const std::os::raw::c_char{
///     string_2_char_strx!(rstr, rstr);/// 
///     rstr
/// }
/// ```
macro_rules! string_2_cstr{
    ($rstr: expr, $name:ident) => {
        let $name = CString::new($rstr).unwrap();
        let $name = $name.as_ptr();
    };
}