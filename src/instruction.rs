use crate::core::*;
use crate::decoder::*;
use crate::fpu_emulator::*;
use crate::types::*;
use crate::utils::*;

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

pub fn exec_instruction(inst: Instruction, core: &mut Core) {
    match inst {
        Instruction::I(imm, rs1, funct3, rd, op) => {
            exec_i_instruction(imm, rs1, funct3, rd, op, core);
        }
        Instruction::R(funct7, rs2, rs1, funct3, rd, op) => {
            exec_r_instruction(funct7, rs2, rs1, funct3, rd, op, core);
        }
        Instruction::S(imm, rs2, rs1, funct3, op) => {
            exec_s_instruction(imm, rs2, rs1, funct3, op, core);
        }
        Instruction::B(imm, rs2, rs1, funct3, op) => {
            create_b_instruction_struct(imm, rs2, rs1, funct3, op, core);
        }
        Instruction::J(imm, rd, op) => {
            exec_j_instruction(imm, rd, op, core);
        }
        Instruction::U(imm, rd, op) => {
            exec_u_instruction(imm, rd, op, core);
        }
        Instruction::R4(fs3, funct2, fs2, fs1, funct3, rd, op) => {
            exec_r4_instruction(fs3, funct2, fs2, fs1, funct3, rd, op, core);
        }
        Instruction::Other => {
            panic!("unexpected instruction: {:?}", inst);
        }
    }
}

pub fn exec_i_instruction(imm: Imm12, rs1: Rs1, funct3: Funct3, rd: Rd, op: Op, core: &mut Core) {
    match op {
        3 => match funct3 {
            0b010 => {
                // lw
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                let value = core.load_word(addr) as Int;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        19 => match funct3 {
            0b000 => {
                // addi
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                let value = core.get_int_register(rs1 as usize) + extended_imm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
            }
            0b001 => {
                // slli
                let uimm = (imm & 0x1f) as u32;
                let value = core.get_int_register(rs1 as usize) << uimm;
                core.set_int_register(rd as usize, value);
                core.increment_pc();
            }
            0b101 => {
                let funct7 = (imm >> 5) & 0b1111111;
                match funct7 {
                    0b0100000 => {
                        // srai
                        let uimm = (imm & 0x1f) as u32;
                        let value = core.get_int_register(rs1 as usize) >> uimm;
                        core.set_int_register(rd as usize, value);
                        core.increment_pc();
                    }
                    _ => {
                        panic!("unexpected funct7: {}", funct7);
                    }
                }
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        103 => match funct3 {
            0b000 => {
                // jalr
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                let jump_address =
                    (core.get_int_register(rs1 as usize) + (extended_imm << 1)) as Address;
                core.set_int_register(rd as usize, core.get_pc() as Int + 4);
                core.set_pc(jump_address);
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        7 => match funct3 {
            0b010 => {
                // flw
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                let value = FloatingPoint::new(i32_to_u32(core.load_word_fp(addr)));
                core.set_float_register(rd as usize, value);
                core.increment_pc();
            }
            _ => {
                panic!("unexpected funct3: {}", funct3)
            }
        },
        115 => match funct3 {
            0b000 => {
                // end
                core.end();
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
) {
    match op {
        51 => match funct3 {
            0b000 => match funct7 {
                0b0000000 => {
                    // add
                    let value =
                        core.get_int_register(rs1 as usize) + core.get_int_register(rs2 as usize);
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                }
                0b0100000 => {
                    // sub
                    let value =
                        core.get_int_register(rs1 as usize) - core.get_int_register(rs2 as usize);
                    core.set_int_register(rd as usize, value);
                    core.increment_pc();
                }
                _ => {
                    panic!("unexpected funct7: {}", funct7);
                }
            },
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        83 => match funct7 {
            0b0000000 => {
                // fadd
                let value =
                    core.get_float_register(rs1 as usize) + core.get_float_register(rs2 as usize);
                core.set_float_register(rd as usize, value);
                core.increment_pc();
            }
            0b0000100 => {
                // fsub
                let value =
                    core.get_float_register(rs1 as usize) - core.get_float_register(rs2 as usize);
                core.set_float_register(rd as usize, value);
                core.increment_pc();
            }
            0b0001000 => {
                // fmul
                let value =
                    core.get_float_register(rs1 as usize) * core.get_float_register(rs2 as usize);
                core.set_float_register(rd as usize, value);
                core.increment_pc();
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
            }
            0b0101100 => {
                // fsqrt
                let value = sqrt_fp(core.get_float_register(rs1 as usize), core.get_sqrt_map());
                core.set_float_register(rd as usize, value);
                core.increment_pc();
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
                }
                0b001 => {
                    // fsgnjn
                    let value = fp_negative_sign_injection(
                        core.get_float_register(rs1 as usize),
                        core.get_float_register(rs2 as usize),
                    );
                    core.set_float_register(rd as usize, value);
                    core.increment_pc();
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
            }
            0b1101000 => {
                // fcvt.s.w
                let value = int_to_fp(core.get_int_register(rs1 as usize));
                core.set_float_register(rd as usize, value);
                core.increment_pc();
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

fn exec_s_instruction(imm: Imm12, rs2: Rs2, rs1: Rs1, funct3: Funct3, op: Op, core: &mut Core) {
    match op {
        35 => match funct3 {
            // sw
            0b010 => {
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                core.store_word(addr, core.get_int_register(rs2 as usize) as Word);
                core.increment_pc();
            }
            _ => {
                panic!("unexpected funct3: {}", funct3);
            }
        },
        39 => match funct3 {
            // fsw
            0b010 => {
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                let addr = (core.get_int_register(rs1 as usize) + extended_imm) as Address;
                core.store_word(
                    addr,
                    u32_to_i32(core.get_float_register(rs2 as usize).get_32_bits()),
                );
                core.increment_pc();
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

fn create_b_instruction_struct(
    imm: Imm12,
    rs2: Rs2,
    rs1: Rs1,
    funct3: Funct3,
    op: Op,
    core: &mut Core,
) {
    match op {
        99 => match funct3 {
            0b000 => {
                // beq
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                if core.get_int_register(rs1 as usize) == core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 1) as Address);
                } else {
                    core.increment_pc();
                }
            }
            0b001 => {
                // bne
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                if core.get_int_register(rs1 as usize) != core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 1) as Address);
                } else {
                    core.increment_pc();
                }
            }
            0b100 => {
                // blt
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                if core.get_int_register(rs1 as usize) < core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 1) as Address);
                } else {
                    core.increment_pc();
                }
            }
            0b101 => {
                // bge
                let extended_imm = sign_extention_i16(imm, 12) as i32;
                if core.get_int_register(rs1 as usize) >= core.get_int_register(rs2 as usize) {
                    core.set_pc(core.get_pc() + (extended_imm << 1) as Address);
                } else {
                    core.increment_pc();
                }
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

fn exec_j_instruction(imm: Imm20, rd: Rd, op: Op, core: &mut Core) {
    match op {
        111 => {
            // jal
            let extended_imm = sign_extention_i32(imm, 20);
            let jump_address = (core.get_pc() as i32 + (extended_imm << 1)) as Address;
            core.set_int_register(rd as usize, core.get_pc() as Int + 4);
            core.set_pc(jump_address);
        }
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

fn exec_u_instruction(imm: Imm20, rd: Rd, op: Op, core: &mut Core) {
    match op {
        55 => {
            let upimm = imm << 12;
            let value = upimm;
            core.set_int_register(rd as usize, value);
            core.increment_pc();
        }
        _ => {
            panic!("unexpected op: {}", op);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn exec_r4_instruction(
    _fs3: Fs3,
    _funct2: Funct2,
    _fs2: Fs2,
    _fs1: Fs1,
    _funct3: Funct3,
    _rd: Rd,
    op: Op,
    _core: &mut Core,
) {
    panic!("unexpected op: {}", op);
}
