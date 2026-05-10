use core::mem::size_of_val;

pub fn sign_extend(x: i32, nbits: u32) -> i32 {
	let notherbits = size_of_val(&x) as u32 * 8 - nbits;
  	x.wrapping_shl(notherbits).wrapping_shr(notherbits)
}

macro_rules! instr_field {
    ($instr_type:ty, $field:ident, $offset:expr, $width:expr) => {
        paste::paste! {
            /// Offset of the $field field in the R-type instruction.
            pub const [<$instr_type:upper _ $field:upper _OFFSET>]: u32 = $offset;
            /// Width of the $field field in the R-type instruction.
            pub const [<$instr_type:upper _ $field:upper _WIDTH>]: u32 = $width;
            /// Mask for the $field field in the R-type instruction, used to extract the field value from the raw instruction word.
            pub const [<$instr_type:upper _ $field:upper _MASK>]: u32 = ((1 << [<$instr_type:upper _ $field:upper _WIDTH>]) - 1) << [<$instr_type:upper _ $field:upper _OFFSET>];
            impl $instr_type {
                /// Extracts the $field field from the instruction, just masking it but without shifting it.
                #[inline(always)]
                pub fn [<raw_ $field>](&self) -> u32 {
                    self.0 & [<$instr_type:upper _ $field:upper _MASK>]
                }
                /// Extracts the $field field from the instruction.
                #[inline(always)]
                pub fn $field(&self) -> u32 {
                    self.[<raw_ $field>]() >> [<$instr_type:upper _ $field:upper _OFFSET>]
                }
            }
        }
    };
}

pub(crate) use instr_field;