#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EMontagePlayReturnType{
	MontageLength = 0,
	Duration = 1,
}