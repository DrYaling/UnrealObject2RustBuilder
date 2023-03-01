
use glam::{Quat, Vec3};
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entity {
    pub id: u64,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// cbindgen:ignore
impl Color {
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Uuid {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

pub type Vector4 = Quaternion;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HitResult {
    pub actor: *mut c_void,
    pub primtive: *mut c_void,
    pub distance: f32,
    pub normal: Vector3,
    pub location: Vector3,
    pub impact_normal: Vector3,
    pub impact_location: Vector3,
    pub pentration_depth: f32,
    pub start_penetrating: u32
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct CollisionBox {
    pub half_extent_x: f32,
    pub half_extent_y: f32,
    pub half_extent_z: f32,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct CollisionSphere {
    pub radius: f32
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct CollisionCapsule {
    pub radius: f32,
    pub half_height: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union CollisionShapeUnion {
    pub collision_box: CollisionBox,
    pub sphere: CollisionSphere,
    pub capsule: CollisionCapsule,
}
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollisionShapeType{
    Box,
    Capsule,
    Sphere,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CollisionShape {
    pub data: CollisionShapeUnion,
    pub ty: CollisionShapeType,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OverlapResult {
    pub actor: *mut c_void,
    pub primtive: *mut c_void,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct SoundSettings {
    pub volume: f32,
    pub pitch: f32,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Rotator {
    /** Rotation around the right axis (around Y axis), Looking up and down (0=Straight Ahead, +Up, -Down) */
    pub pitch: f32,

    /** Rotation around the up axis (around Z axis), Turning around (0=Forward, +Right, -Left)*/
    pub yaw: f32,

    /** Rotation around the forward axis (around X axis), Tilting your head, (0=Straight, +Clockwise, -CCW) */
    pub roll: f32,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntPoint
{
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Transform
{
    pub rotation: Quaternion,
    pub location: Vector3,
    pub scale: Vector3
}

impl From<Quaternion> for Quat {
    fn from(val: Quaternion) -> Self {
        Quat::from_xyzw(val.x, val.y, val.z, val.w)
    }
}

impl From<Vector3> for Vec3 {
    fn from(val: Vector3) -> Self {
        Vec3::new(val.x, val.y, val.z)
    }
}

impl From<Vec3> for Vector3 {
    fn from(v: Vec3) -> Self {
        Vector3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}
impl From<Quat> for Quaternion {
    fn from(v: Quat) -> Self {
        Quaternion {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}
/*
unreal FName
 */
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct UName{
	pub entry: u32,
	pub number: u32,
}
///imply that this is a unreal object
pub trait IPtr: Sized{
    fn inner(&self) -> *mut c_void;
    fn from_ptr(ptr: *mut c_void) -> Option<Self>;
}
///cast V into R, this casting is unsafe, user should ensure the safety
pub unsafe fn cast_to<V: IPtr, R: IPtr>(from: V) -> Option<R>{
    R::from_ptr(from.inner())
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
pub struct NativeString{
    pub utf_str: *const c_char,
    pub size: u32,
}
///thread unsafe
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RefString{
    pub utf_str: *const c_char,
    pub str_ref: *mut String,
    pub size: u32,
}
/// rust string to c const string(as parameter)
/// ```
/// pub fn string_2_char_str(rstr: &str) -> binders::NativeString{
///     string_2_cstr!(rstr, rstr);/// 
///     rstr
/// }
/// ```
#[allow(unused)]
#[macro_export]
macro_rules! string_2_cstr{
    ($rstr: expr, $name:ident) => {
        let size = $rstr.len() as u32;
        let utf_str = $rstr.as_ptr() as *const std::ffi::c_char;
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
#[allow(unused)]
#[macro_export]
macro_rules! string_2_rstr{
    ($rstr: expr, $name:ident) => {
        let str_ref = $rstr as *mut String;
        let size = $rstr.len() as u32;
        let utf_str = $rstr.as_str().as_ptr() as *const std::ffi::c_char;
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
