
#[repr(C)]
pub struct FColor{
    pub B: u8,
    pub G: u8,
    pub R: u8,
    pub A: u8,
}
#[repr(C)]
pub struct FLinearColor{
	pub R: f32,
	pub G: f32,
	pub B: f32,
	pub A: f32
}
#[repr(C)]
pub struct FIntVector{
	pub X: i32,
	pub Y: i32,
	pub Z: i32
}
#[repr(C)]
pub struct FIntPoint{
	pub X: i32,
	pub Y: i32
}