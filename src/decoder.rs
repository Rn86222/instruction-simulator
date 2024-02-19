use crate::types::*;

const REG_MASK: u32 = 63;
const FUNCT3_MASK: u32 = 7;
const OP_MASK: u32 = 15;

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    I(Imm13, Rs1, Funct3, Rd, Op),
    R(Funct7, Rs2, Rs1, Funct3, Rd, Op),
    S(Imm13, Rs2, Rs1, Funct3, Op),
    J(Imm19, Rd, Op),
    B(Imm13, Rs2, Rs1, Funct3, Op),
    U(Imm19, Rd, Op),
    Other,
}

enum InstructionType {
    I,
    R,
    S,
    J,
    B,
    U,
    Other,
}

fn instruction_typeof(inst: InstructionValue) -> InstructionType {
    if inst == 0 {
        return InstructionType::Other;
    }
    let op = inst & OP_MASK;
    match op {
        0 | 1 | 6 | 8 | 11 | 14 => InstructionType::I,
        3 | 9 => InstructionType::R,
        2 | 10 | 12 => InstructionType::S,
        7 => InstructionType::J,
        5 | 13 => InstructionType::B,
        4 => InstructionType::U,
        _ => InstructionType::Other,
    }
}

fn decode_i_instruction(inst: InstructionValue) -> Instruction {
    let imm: i16 = (inst >> 19) as i16;
    let rs1 = ((inst >> 13) & REG_MASK) as u8;
    let funct3 = ((inst >> 10) & FUNCT3_MASK) as u8;
    let rd = ((inst >> 4) & REG_MASK) as u8;
    let op = (inst & OP_MASK) as u8;
    Instruction::I(imm, rs1, funct3, rd, op)
}

fn decode_r_instruction(inst: InstructionValue) -> Instruction {
    let funct7 = (inst >> 25) as u8;
    let rs2 = ((inst >> 19) & REG_MASK) as u8;
    let rs1 = ((inst >> 13) & REG_MASK) as u8;
    let funct3 = ((inst >> 10) & FUNCT3_MASK) as u8;
    let rd = ((inst >> 4) & REG_MASK) as u8;
    let op = (inst & OP_MASK) as u8;
    Instruction::R(funct7, rs2, rs1, funct3, rd, op)
}

fn decode_s_instruction(inst: InstructionValue) -> Instruction {
    let imm: i16 = ((inst >> 25) << 6) as i16 + (((inst >> 4) & 63) as i16);
    let rs2 = ((inst >> 19) & REG_MASK) as u8;
    let rs1 = ((inst >> 13) & REG_MASK) as u8;
    let funct3 = ((inst >> 10) & FUNCT3_MASK) as u8;
    let op = (inst & OP_MASK) as u8;
    Instruction::S(imm, rs2, rs1, funct3, op)
}

fn decode_j_instruction(inst: InstructionValue) -> Instruction {
    let imm: i32 = ((inst >> 31) << 18) as i32
        + (((inst >> 13) & 255) << 10) as i32
        + (((inst >> 21) & 1) << 9) as i32
        + ((inst >> 22) & 511) as i32;
    let rd = ((inst >> 4) & REG_MASK) as u8;
    let op = (inst & OP_MASK) as u8;
    Instruction::J(imm, rd, op)
}

fn decode_b_instruction(inst: InstructionValue) -> Instruction {
    let imm: i16 = ((inst >> 31) << 12) as i16
        + (((inst >> 4) & 1) << 11) as i16
        + (((inst >> 25) & 63) << 5) as i16
        + ((inst >> 5) & 31) as i16;
    let rs2 = ((inst >> 19) & REG_MASK) as u8;
    let rs1 = ((inst >> 13) & REG_MASK) as u8;
    let funct3 = ((inst >> 10) & FUNCT3_MASK) as u8;
    let op = (inst & OP_MASK) as u8;
    Instruction::B(imm, rs2, rs1, funct3, op)
}

fn decode_u_instruction(inst: InstructionValue) -> Instruction {
    let imm: i32 = (inst >> 13) as i32;
    let rd = ((inst >> 4) & REG_MASK) as u8;
    let op = (inst & OP_MASK) as u8;
    Instruction::U(imm, rd, op)
}

pub fn decode_instruction(inst: InstructionValue) -> Instruction {
    let instruction_type = instruction_typeof(inst);
    match instruction_type {
        InstructionType::I => decode_i_instruction(inst),
        InstructionType::R => decode_r_instruction(inst),
        InstructionType::S => decode_s_instruction(inst),
        InstructionType::J => decode_j_instruction(inst),
        InstructionType::B => decode_b_instruction(inst),
        InstructionType::U => decode_u_instruction(inst),
        InstructionType::Other => Instruction::Other,
    }
}
