use core::mem::size_of_val;

pub fn sign_extend(x: i32, nbits: u32) -> i32 {
	let notherbits = size_of_val(&x) as u32 * 8 - nbits;
  	x.wrapping_shl(notherbits).wrapping_shr(notherbits)
}
