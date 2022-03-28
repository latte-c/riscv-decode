use crate::{DecodingResult, DecodingError};
use crate::types::*;
use crate::instruction::*;

/* 
pub fn decode_q00(i: u32) -> DecodingResult {
    match i >> 13 {
        0b000 if i == 0 => Ok(Instruction::Illegal),
        0b000 => Err(DecodingError::Unimplemented), // C.ADDI4SPN
        0b001 => Err(DecodingError::Unimplemented), // C.FLD
        0b010 => Ok(Instruction::Lw(IType(
            ((i & 0x1c00) << 13)      // imm[5:3]
            | ((i & 0x380) << 8)      // rs1[2:0]
            | ((i & 0x40) << 16)      // imm[2]
            | ((i & 0x20) << 21)      // imm[6]
            | ((i & 0x1c) << 5)       // rd[2:0]
            | 0b_01000_010_01000_0000011,
        ))),
        0b011 => Ok(Instruction::Ld(IType(
            // C.LD (C.FLW in RV32)
            ((i & 0x1c00) << 13)      // imm[5:3]
            | ((i & 0x380) << 8)      // rs1[2:0]
            | ((i & 0x60) << 21)      // imm[7:6]
            | ((i & 0x1c) << 5)       // rd[2:0]
            | 0b_01000_011_01000_0000011,
        ))),
        0b100 => Err(DecodingError::Unimplemented), // reserved
        0b101 => Err(DecodingError::Unimplemented), // C.FSD
        0b110 => Ok(Instruction::Sw(SType(
            // C.SW
            ((i & 0x1000) << 13)      // imm[5]
            | ((i & 0xc00))           // imm[4:3]
            | ((i & 0x380) << 8)      // rs1[2:0]
            | ((i & 0x40) << 3)       // imm[2]
            | ((i & 0x20) << 21)      // imm[6]
            | ((i & 0x1c) << 18)      // rs2[2:0]
            | 0b_01000_01000_010_00000_0100011,
        ))),
        0b111 => Ok(Instruction::Sd(SType(
            // C.SD (C.FSW in RV32)
            ((i & 0x1000) << 13)      // imm[5]
            | ((i & 0xc00))           // imm[4:3]
            | ((i & 0x380) << 8)      // rs1[2:0]
            | ((i & 0x60) << 21)      // imm[7:6]
            | ((i & 0x1c) << 18)      // rs2[2:0]
            | 0b_01000_01000_011_00000_0100011,
        ))),
        _ => Err(DecodingError::Unimplemented),
    }
}
*/

type CInsn = CompressedInstruction;

const fn c_insn(i: CInsn) -> DecodingResult {
    Ok(Instruction::Compressed(i))
}

const fn unimpl() -> DecodingResult {
    Err(DecodingError::Unimplemented)
}

pub fn decode_q00(i: u16) -> DecodingResult {
    match i >> 13 {
        0b000 if i == 0 => Ok(Instruction::Illegal), // illegal
        0b000 => c_insn(CInsn::CAddi4spn(CIWType(i))), // C.ADDI4SPN
        0b001 => unimpl(), // C.FLD / C.LQ
        0b010 => c_insn(CInsn::CLw(CLType(i))),     // C.LW
        0b011 => unimpl(), // C.FLW / C.LD
        0b100 => Err(DecodingError::Reserved),
        0b101 => unimpl(), // C.FSD / C.SQ
        0b110 => c_insn(CInsn::CSw(CSType(i))), // C.SW
        0b111 => unimpl(), // C.FSW / C.SD 
        _ => unreachable!(),
    }
}

pub fn decode_q01(i: u16) -> DecodingResult {
    match i >> 13 {
        0b000 => {
            let rd = (i >> 7) & 0x1f;
            match rd {
                0u16 => c_insn(CInsn::CNop), // C.NOP
                _ => c_insn(CInsn::CAddi(CIType(i))), // C.ADDI
            }
        },
        0b001 => unimpl(), // C.JAL / C.ADDIW
        0b010 => c_insn(CInsn::CLi(CIType(i))), // C.LI
        0b011 => {
            let rd = (i >> 7) & 0x1f;
            match rd {
                2u16 => c_insn(CInsn::CAddi16sp(CIType(i))), // C.ADDI16SP
                _ => c_insn(CInsn::CLui(CIType(i))), // C.LUI
            }
        },
        0b100 => { // misc ALU
            let f1 = (i >> 10) & 0b11; // [11:10]
            match f1 {
                0b00 => unimpl(), // C.SRLI / C.SRLI64
                0b01 => unimpl(), // C.SRAI / C.SRAI64
                0b10 => c_insn(CInsn::CAndi(CBType(i))),
                0b11 => {
                    let f2 = ((i >> 10) & 0b100) | ((i >> 5) & 0b11); // [12] # [6:5]
                    match f2 {
                        0b000 => c_insn(CInsn::CSub(CAType(i))),  // C.SUB
                        0b001 => c_insn(CInsn::CXor(CAType(i))),  // C.XOR
                        0b010 => c_insn(CInsn::COr(CAType(i))),   // C.OR
                        0b011 => c_insn(CInsn::CAnd(CAType(i))),  // C.AND
                        0b100 => c_insn(CInsn::CSubw(CAType(i))), // C.SUBW
                        0b101 => c_insn(CInsn::CAddw(CAType(i))), // C.ADDW
                        _ => Err(DecodingError::Reserved),
                    }
                },
                _ => unreachable!(),
            }
        },
        0b101 => c_insn(CInsn::CJ(CJType(i))),    // C.J
        0b110 => c_insn(CInsn::CBeqz(CBType(i))), // C.BEQZ
        0b111 => c_insn(CInsn::CBnez(CBType(i))), // C.BNEZ
        _ => unreachable!(),
    }
}

pub fn decode_q10(i: u16) -> DecodingResult {
    match i >> 13 {
        0b000 => unimpl(), // C.SLLI / C.SLLI64
        0b001 => unimpl(), // C.FLDSP / C.LQSP
        0b010 => c_insn(CInsn::CLwsp(CIType(i))), // C.LWSP
        0b011 => unimpl(), // C.FLWSP / C.LDSP
        0b100 => {
            let zero_rd = ((i >> 7) & 0x1f) == 0u16; // rd/rs1
            let zero_rs2 = ((i >> 2) & 0x1f) == 0u16;
            match (i >> 12) & 1 {
                0u16 => {
                    match zero_rs2 {
                        true => c_insn(CInsn::CJr(CRType(i))),  // C.JR
                        false => c_insn(CInsn::CMv(CRType(i))), // C.MV
                    }
                },
                _ => {
                    match zero_rs2 {
                        true => {
                            match zero_rd {
                                true => c_insn(CInsn::CEbreak),
                                false => c_insn(CInsn::CJalr(CRType(i))),
                            }
                        },
                        false => c_insn(CInsn::CAdd(CRType(i))),
                    }
                }
            }
        },
        0b101 => unimpl(), // C.FSDSP / C.SQSP
        0b110 => c_insn(CInsn::CSwsp(CSSType(i))),
        0b111 => unimpl(), // C.FSWSP / C.SDSP
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    // use super::Instruction::*;
    use super::CompressedInstruction::*;
    use super::*;

    /*
    #[test]
    fn q00() {
        assert_eq!(decode_q00(0x6188).unwrap(), Ld(IType(0x0005b503))); // ld a0,0(a1)
        assert_eq!(decode_q00(0x75e0).unwrap(), Ld(IType(0x0e85b403))); // ld s0,232(a1)
        assert_eq!(decode_q00(0x43b0).unwrap(), Lw(IType(0x0407a603))); // lw a2,64(a5)
        assert_eq!(decode_q00(0xe188).unwrap(), Sd(SType(0x00a5b023))); // sd a0,0(a1)
        assert_eq!(decode_q00(0xf5e0).unwrap(), Sd(SType(0x0e85b423))); // sd s0,232(a1)
        assert_eq!(decode_q00(0xc3b0).unwrap(), Sw(SType(0x04c7a023))); // sw a2,64(a5)
    }
    */

    #[test]
    fn compressed() {
        assert_eq!(decode_q01(0x7179), c_insn(CAddi16sp(CIType(0x7179)))); // addi sp,sp,-48
        assert_eq!(decode_q10(0x892a), c_insn(CMv(CRType(0x892a))));   // mv s2,a0
        assert_eq!(decode_q01(0x4585), c_insn(CLi(CIType(0x4585))));   // li a1,1
        assert_eq!(decode_q10(0x8082), c_insn(CJr(CRType(0x8082))));   // ret
    }
}
