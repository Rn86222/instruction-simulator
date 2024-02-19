// use std::collections::HashMap;

use crate::core::*;
use crate::decoder::*;
use crate::fpu_emulator::*;
use crate::types::*;
use crate::utils::*;

const FADD_STALL: usize = 2;
const FSUB_STALL: usize = 2;
const FMUL_STALL: usize = 2;
const FDIV_STALL: usize = 10;
const FSQRT_STALL: usize = 7;
const FLT_STALL: usize = 0;
const FEQ_STALL: usize = 0;
const FLE_STALL: usize = 0;
const FCVTSW_STALL: usize = 1;
const FCVTWS_STALL: usize = 1;

#[derive(Clone, Copy)]
pub enum InstructionId {
    Lw,
    Addi,
    Slli,
    Slti,
    Xori,
    Srli,
    Srai,
    Ori,
    Andi,
    Sw,
    Add,
    Sub,
    Sll,
    Slt,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    Lui,
    Beq,
    Bne,
    Blt,
    Bge,
    Jalr,
    Jal,
    Flw,
    Fadd,
    Fsub,
    Fmul,
    Fdiv,
    Fsqrt,
    Fsgnj,
    Fsgnjn,
    Fsgnjx,
    Feq,
    Flt,
    Fle,
    FcvtWS,
    FcvtSW,
    Fsw,
    In,
    Fin,
    Outchar,
    Outint,
    Fbeq,
    Fbne,
    Fblt,
    Fble,
    End,
}

pub const INST_ID_TO_NAME: [&str; 50] = [
    "lw", "addi", "slli", "slti", "xori", "srli", "srai", "ori", "andi", "sw", "add", "sub", "sll",
    "slt", "xor", "srl", "sra", "or", "and", "lui", "beq", "bne", "blt", "bge", "jalr", "jal",
    "flw", "fadd", "fsub", "fmul", "fdiv", "fsqrt", "fsgnj", "fsgnjn", "fsgnjx", "feq", "flt",
    "fle", "fcvt.w.s", "fcvt.s.w", "fsw", "in", "fin", "outchar", "outint", "fbeq", "fbne", "fblt",
    "fble", "end",
];

const UIMM_MASK: i16 = 63;

pub fn sign_extention_i16(value: i16, before_bit: usize) -> i16 {
    if (value >> (before_bit - 1)) & 1 == 0 {
        value
    } else {
        let mut extention: i16 = 0;
        for i in 0..16 - before_bit {
            extention += 1 << (before_bit + i);
        }
        value | extention
    }
}

pub fn sign_extention_i32(value: i32, before_bit: usize) -> i32 {
    if (value >> (before_bit - 1)) & 1 == 0 {
        value
    } else {
        let mut extention: i32 = 0;
        for i in 0..32 - before_bit {
            extention += 1 << (before_bit + i);
        }
        value | extention
    }
}

pub fn exec_instruction(inst: Instruction, core: &mut Core) -> InstructionId {
    match inst {
        Instruction::I(imm, rs1, funct3, rd, op) => {
            exec_i_instruction(imm, rs1, funct3, rd, op, core)
        }
        Instruction::R(funct7, rs2, rs1, funct3, rd, op) => {
            exec_r_instruction(funct7, rs2, rs1, funct3, rd, op, core)
        }
        Instruction::S(imm, rs2, rs1, funct3, op) => {
            exec_s_instruction(imm, rs2, rs1, funct3, op, core)
        }
        Instruction::B(imm, rs2, rs1, funct3, op) => {
            exec_b_instruction(imm, rs2, rs1, funct3, op, core)
        }
        Instruction::J(imm, rd, op) => exec_j_instruction(imm, rd, op, core),
        Instruction::U(imm, rd, op) => exec_u_instruction(imm, rd, op, core),
        Instruction::Other => {
            panic!("unexpected instruction: {:?}", inst);
        }
    }
}

pub fn exec_i_instruction(
    imm: Imm13,
    rs1: Rs1,
    funct3: Funct3,
    rd: Rd,
    op: Op,
    core: &mut Core,
) -> InstructionId {
    match op {
        0 => match funct3 {
            0b010 => {
                // lw
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                let value = core.load_word(addr) as Int;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                core.set_load_dest(rd as usize);
                InstructionId::Lw
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        1 => match funct3 {
            0b000 => {
                // addi
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let value = core.get_int_register(rs1 as usize) + extended_imm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Addi
            }
            0b001 => {
                // slli
                let uimm = (imm & UIMM_MASK) as u32;
                let value = core.get_int_register(rs1 as usize) << uimm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Slli
            }
            0b010 => {
                // slti
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let value = if core.get_int_register(rs1 as usize) < extended_imm {
                    1
                } else {
                    0
                };
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Slti
            }
            0b100 => {
                // xori
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let value = core.get_int_register(rs1 as usize) ^ extended_imm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Xori
            }
            0b101 => {
                let funct7 = (imm >> 6) & 0b1111111;
                match funct7 {
                    0b0000000 => {
                        // srli
                        let uimm = (imm & UIMM_MASK) as u32;
                        let value =
                            u32_to_i32(i32_to_u32(core.get_int_register(rs1 as usize)) >> uimm);
                        core.set_int_register(rd as usize, value);
                        core.increment_pc();
                        InstructionId::Srli
                    }
                    0b0100000 => {
                        // srai
                        let uimm = (imm & UIMM_MASK) as u32;
                        let value = core.get_int_register(rs1 as usize) >> uimm;
                        core.set_int_register(rd as usize, value);
                        core.increment_pc();
                        InstructionId::Srai
                    }
                    _ => {
                        panic!("unexpected funct7: {}", funct7);
                    }
                }
            }
            0b110 => {
                // ori
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let value = core.get_int_register(rs1 as usize) | extended_imm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Ori
            }
            0b111 => {
                // andi
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let value = core.get_int_register(rs1 as usize) & extended_imm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Andi
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        6 => match funct3 {
            0b000 => {
                // jalr
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let jump_address =
                    (core.get_int_register(rs1 as usize) + (extended_imm << 2)) as Address;
                core.set_int_register(rd as usize, core.get_pc() as Int + 4);
                core.set_pc(jump_address);
                core.increment_flush_counter();
                InstructionId::Jalr
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        8 => match funct3 {
            0b010 => {
                // flw
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                let value = FloatingPoint::new(i32_to_u32(core.load_word(addr)));
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.set_load_dest(rd as usize + 32);
                InstructionId::Flw
            }
            _ => {
                panic!("unexpected funct3: {}", funct3)
            }
        },
        11 => match funct3 {
            0b000 => {
                // in
                let value = core.read_int();
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::In
            }
            0b001 => {
                // fin
                let value = core.read_float();
                core.set_float_register(rd as usize, FloatingPoint::new(i32_to_u32(value)));
                core.increment_pc();
                InstructionId::Fin
            }
            _ => {
                panic!("unexpected funct3: {}", funct3)
            }
        },
        14 => match funct3 {
            0b000 => {
                // end
                core.end();
                InstructionId::End
            }
            _ => {
                panic!("unexpected funct3: {}", funct3)
            }
        },
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

fn exec_r_instruction(
    funct7: Funct7,
    rs2: Rs2,
    rs1: Rs2,
    funct3: Funct3,
    rd: Rd,
    op: Op,
    core: &mut Core,
) -> InstructionId {
    match op {
        3 => match funct3 {
            0b000 => match funct7 {
                0b0000000 => {
                    // add
                    let value =
                        core.get_int_register(rs1 as usize) + core.get_int_register(rs2 as usize);
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Add
                }
                0b0100000 => {
                    // sub
                    let value =
                        core.get_int_register(rs1 as usize) - core.get_int_register(rs2 as usize);
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Sub
                }
                _ => {
                    panic!("unexpected funct7: {}", funct7);
                }
            },
            0b001 => {
                // sll
                let shamt = core.get_int_register(rs2 as usize) & 0b11111;
                let value = core.get_int_register(rs1 as usize) << shamt;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Sll
            }
            0b010 => {
                // slt
                let value =
                    if core.get_int_register(rs1 as usize) < core.get_int_register(rs2 as usize) {
                        1
                    } else {
                        0
                    };
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Slt
            }
            0b100 => match funct7 {
                0b0000000 => {
                    // xor
                    let value =
                        core.get_int_register(rs1 as usize) ^ core.get_int_register(rs2 as usize);
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Xor
                }
                _ => {
                    panic!("unexpected funct7: {}", funct7);
                }
            },
            0b101 => match funct7 {
                0b0000000 => {
                    // srl
                    let shamt = core.get_int_register(rs2 as usize) & 0b11111;
                    let value =
                        u32_to_i32(i32_to_u32(core.get_int_register(rs1 as usize)) >> shamt);
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Srl
                }
                0b0100000 => {
                    // sra
                    let shamt = core.get_int_register(rs2 as usize) & 0b11111;
                    let value = core.get_int_register(rs1 as usize) >> shamt;
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Sra
                }
                _ => {
                    panic!("unexpected funct7: {}", funct7);
                }
            },
            0b110 => {
                // or
                let value =
                    core.get_int_register(rs1 as usize) | core.get_int_register(rs2 as usize);
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::Or
            }
            0b111 => {
                // and
                let value =
                    core.get_int_register(rs1 as usize) & core.get_int_register(rs2 as usize);
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                InstructionId::And
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        9 => match funct7 {
            0b0000000 => {
                // fadd
                let value =
                    core.get_float_register(rs1 as usize) + core.get_float_register(rs2 as usize);
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FADD_STALL);
                InstructionId::Fadd
            }
            0b0000100 => {
                // fsub
                let value =
                    core.get_float_register(rs1 as usize) - core.get_float_register(rs2 as usize);
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FSUB_STALL);
                InstructionId::Fsub
            }
            0b0001000 => {
                // fmul
                let value =
                    core.get_float_register(rs1 as usize) * core.get_float_register(rs2 as usize);
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FMUL_STALL);
                InstructionId::Fmul
            }
            0b0001100 => {
                // fdiv
                let value = div_fp(
                    core.get_float_register(rs1 as usize),
                    core.get_float_register(rs2 as usize),
                    core.get_inv_map(),
                );
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FDIV_STALL);
                InstructionId::Fdiv
            }
            0b0101100 => {
                // fsqrt
                let value = sqrt_fp(core.get_float_register(rs1 as usize), core.get_sqrt_map());
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FSQRT_STALL);
                InstructionId::Fsqrt
            }
            0b0010000 => match funct3 {
                0b000 => {
                    // fsgnj
                    let value = fp_sign_injection(
                        core.get_float_register(rs1 as usize),
                        core.get_float_register(rs2 as usize),
                    );
                    core.set_float_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Fsgnj
                }
                0b001 => {
                    // fsgnjn
                    let value = fp_negative_sign_injection(
                        core.get_float_register(rs1 as usize),
                        core.get_float_register(rs2 as usize),
                    );
                    core.set_float_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Fsgnjn
                }
                0b010 => {
                    // fsgnjx
                    let value = fp_xor_sign_injection(
                        core.get_float_register(rs1 as usize),
                        core.get_float_register(rs2 as usize),
                    );
                    core.set_float_register(rd as usize, value);
                    core.increment_pc();
                    InstructionId::Fsgnjx
                }
                _ => {
                    panic!("unexpected funct3: {}", funct3)
                }
            },
            0b0010100 => {
                panic!("unexpected funct7: {}", funct7)
            }
            0b1010000 => match funct3 {
                0b010 => {
                    // feq
                    let value = if core.get_float_register(rs1 as usize)
                        == core.get_float_register(rs2 as usize)
                    {
                        1
                    } else {
                        0
                    };
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    core.increment_fpu_stall_counter(FEQ_STALL);
                    InstructionId::Feq
                }
                0b001 => {
                    // flt
                    let value = if core.get_float_register(rs1 as usize)
                        < core.get_float_register(rs2 as usize)
                    {
                        1
                    } else {
                        0
                    };
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    core.increment_fpu_stall_counter(FLT_STALL);
                    InstructionId::Flt
                }
                0b000 => {
                    // fle
                    let value = if core.get_float_register(rs1 as usize)
                        <= core.get_float_register(rs2 as usize)
                    {
                        1
                    } else {
                        0
                    };
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                    core.increment_fpu_stall_counter(FLE_STALL);
                    InstructionId::Fle
                }
                _ => {
                    panic!("unexpected funct3: {}", funct3)
                }
            },
            0b1100000 => {
                // fcvt.w.s
                let value = fp_to_int(core.get_float_register(rs1 as usize));
                core.set_int_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FCVTWS_STALL);
                InstructionId::FcvtWS
            }
            0b1101000 => {
                // fcvt.s.w
                let value = int_to_fp(core.get_int_register(rs1 as usize));
                core.set_float_register(rd as usize, value);
                core.increment_pc();
                core.increment_fpu_stall_counter(FCVTSW_STALL);
                InstructionId::FcvtSW
            }
            _ => {
                panic!("unexpected funct7: {}", funct7)
            }
        },
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

fn exec_s_instruction(
    imm: Imm13,
    rs2: Rs2,
    rs1: Rs1,
    funct3: Funct3,
    op: Op,
    core: &mut Core,
) -> InstructionId {
    match op {
        2 => match funct3 {
            // sw
            0b010 => {
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                let rs2_value = core.get_int_register(rs2 as usize);
                core.store_word(addr, rs2_value as Word);
                core.increment_pc();
                InstructionId::Sw
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        10 => match funct3 {
            0b010 => {
                // fsw
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                let rs2_value = core.get_float_register(rs2 as usize);
                core.store_word(addr, u32_to_i32(rs2_value.get_32_bits()));
                core.increment_pc();
                InstructionId::Fsw
            }
            _ => {
                panic!("unexpected funct3: {}", funct3)
            }
        },
        12 => match funct3 {
            0b000 => {
                // outchar
                let value = core.get_int_register(rs2 as usize);
                core.print_char(value);
                core.increment_pc();
                InstructionId::Outchar
            }
            0b001 => {
                // outint
                let value = core.get_int_register(rs2 as usize);
                core.print_int(value);
                core.increment_pc();
                InstructionId::Outint
            }
            _ => {
                panic!("unexpected funct3: {}", funct3)
            }
        },
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

fn exec_b_instruction(
    imm: Imm13,
    rs2: Rs2,
    rs1: Rs1,
    funct3: Funct3,
    op: Op,
    core: &mut Core,
) -> InstructionId {
    match op {
        5 => match funct3 {
            0b000 => {
                // beq
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_int_register(rs1 as usize) == core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Beq
            }
            0b001 => {
                // bne
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_int_register(rs1 as usize) != core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Bne
            }
            0b100 => {
                // blt
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_int_register(rs1 as usize) < core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Blt
            }
            0b101 => {
                // bge
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_int_register(rs1 as usize) >= core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Bge
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        13 => match funct3 {
            0b000 => {
                // fbeq
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_float_register(rs1 as usize) == core.get_float_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Fbeq
            }
            0b001 => {
                // fbne
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_float_register(rs1 as usize) != core.get_float_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Fbne
            }
            0b100 => {
                // fblt
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_float_register(rs1 as usize) < core.get_float_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Fblt
            }
            0b101 => {
                // fble
                let extended_imm = sign_extention_i16(imm, 13) as i32;
                if core.get_float_register(rs1 as usize) <= core.get_float_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 2) as Address);
                } else {
                    core.increment_pc();
                }
                core.increment_flush_counter();
                InstructionId::Fble
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

fn exec_j_instruction(imm: Imm19, rd: Rd, op: Op, core: &mut Core) -> InstructionId {
    match op {
        7 => {
            // jal
            let extended_imm = sign_extention_i32(imm, 19);
            let jump_address = (core.get_pc() as i32 + (extended_imm << 2)) as Address;
            core.set_int_register(rd as usize, core.get_pc() as Int + 4);
            core.set_pc(jump_address);
            core.increment_flush_counter();
            InstructionId::Jal
        }
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

fn exec_u_instruction(imm: Imm19, rd: Rd, op: Op, core: &mut Core) -> InstructionId {
    match op {
        4 => {
            // lui
            let upimm = imm << 13;
            let value = upimm;
            core.set_int_register(rd as usize, value);
            core.increment_pc();
            InstructionId::Lui
        }
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}
