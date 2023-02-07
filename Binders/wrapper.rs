
///imply that this is a unreal object
pub trait IPtr{
    fn inner(&self) -> *mut c_void;
    fn from_ptr(ptr: *mut c_void) -> Option<Self>;
}
///cast V into R, this casting is unsafe, user should ensure the safety
pub unsafe fn cast_to<V: IPtr, R: IPtr>(from: V) -> Option<R>{
    R::from_ptr(from.inner())
}
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
    let slice = std::slice::from_raw_parts(ptr as *const u8, size as usize);
    let Ok(native_str) = CString::new(slice)
    else{
        error!("fail to parse from {:p}", ptr);
        return std::ptr::null_mut();
    };
    native_str.into_raw()
}
///thread unsafe
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NativeString{
    utf_str: *const c_char,
    size: u32,
}
///thread unsafe
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RefString{
    utf_str: *const c_char,
    str_ref: *mut String,
    size: u32,
}
/// rust string to c const string(as parameter)
/// ```
/// pub fn string_2_char_str(rstr: &str) -> binders::NativeString{
///     string_2_cstr!(rstr, rstr);/// 
///     rstr
/// }
/// ```
macro_rules! string_2_cstr{
    ($rstr: expr, $name:ident) => {
        let size = $rstr.len() as u32;
        let utf_str = $rstr.as_ptr() as *const c_char;
        let $name = NativeString{utf_str, size};
    };
}
/// rust string to c ref string(as parameter)
/// ```
/// pub fn string_2_rstr(rstr: &mut String) -> binders::RefString{
///     string_2_rstr!(rstr, rstr);/// 
///     rstr
/// }
/// ```
macro_rules! string_2_rstr{
    ($rstr: expr, $name:ident) => {
        let str_ref = $rstr as *mut String;
        let size = $rstr.len() as u32;
        let utf_str = $rstr.as_str().as_ptr() as *const c_char;
        let $name = RefString{utf_str, size, str_ref};
    };
}
#[no_mangle]
unsafe extern fn reset_rust_string(rstr: RefString, c_str: *const c_char, size: u32){
    // info!("rest rust string {:p}", c_str);
    if c_str.is_null(){
        return;
    }
    if let Some(r_str) = rstr.str_ref.as_mut(){
        if size > rstr.size{
            r_str.reserve_exact((size - rstr.size) as usize + 1);
        }
        r_str.as_mut_vec().set_len(size as usize);
        std::intrinsics::copy(c_str as *const u8, r_str.as_mut_vec().as_mut_ptr(), size as usize);
    }
}
