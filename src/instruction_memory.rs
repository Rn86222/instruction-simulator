use crate::types::*;
pub const INSTRUCTION_MEMORY_SIZE: usize = 4 * 1024 * 1024;

pub struct InstructionMemory {
    values: [InstructionValue; INSTRUCTION_MEMORY_SIZE],
}

impl InstructionMemory {
    pub fn new() -> Self {
        let init_val = 0;
        let values = [init_val; INSTRUCTION_MEMORY_SIZE];
        InstructionMemory { values }
    }

    pub fn load(&self, addr: Address) -> InstructionValue {
        self.values[(addr >> 2) as usize]
    }

    pub fn store(&mut self, addr: Address, value: InstructionValue) {
        self.values[(addr >> 2) as usize] = value;
    }
}
