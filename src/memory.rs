use crate::cache::LINE_SIZE;
use crate::types::*;
use crate::utils::*;
pub const MEMORY_SIZE: usize = 128 * 1024 * 1024;

pub struct Memory {
    values: [MemoryValue; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        let init_val = 0;
        let values = [init_val; MEMORY_SIZE];
        Memory { values }
    }

    #[allow(dead_code)]
    pub fn load_byte(&self, addr: Address) -> Byte {
        u8_to_i8(self.values[addr as usize]) as Byte
    }

    pub fn load_ubyte(&self, addr: Address) -> UByte {
        self.values[addr as usize] as UByte
    }

    pub fn store_byte(&mut self, addr: Address, value: Byte) {
        self.values[addr as usize] = i8_to_u8(value);
    }

    fn store_ubyte(&mut self, addr: Address, value: UByte) {
        self.values[addr as usize] = value;
    }

    #[allow(dead_code)]
    pub fn load_half(&self, addr: Address) -> Half {
        let mut load_value: u16 = 0;

        for i in 0..2 {
            load_value += (self.load_ubyte(addr + i) as u16) << (8 * i);
        }
        u16_to_i16(load_value) as Half
    }

    #[allow(dead_code)]
    pub fn load_uhalf(&self, addr: Address) -> UHalf {
        let mut load_value: u16 = 0;

        for i in 0..2 {
            load_value += (self.load_ubyte(addr + i) as u16) << (8 * i);
        }
        load_value as UHalf
    }

    pub fn load_word(&self, addr: Address) -> Word {
        let mut load_value: u32 = 0;
        for i in 0..4 {
            load_value += (self.load_ubyte(addr + i) as u32) << (8 * i);
        }
        u32_to_i32(load_value) as Word
    }

    #[allow(dead_code)]
    pub fn store_half(&mut self, addr: Address, value: Half) {
        for i in 0..2 {
            self.store_byte(addr + i, ((value >> (i * 8)) & 0xff) as Byte);
        }
    }

    pub fn store_word(&mut self, addr: Address, value: Word) {
        for i in 0..4 {
            self.store_byte(addr + i, ((value >> (i * 8)) & 0xff) as Byte);
        }
    }

    pub fn get_cache_line(&self, addr: Address) -> [MemoryValue; LINE_SIZE] {
        let mut line = [0; LINE_SIZE];
        for (i, value) in line.iter_mut().enumerate() {
            *value = self.load_ubyte(addr + i as Address);
        }
        line
    }

    pub fn set_cache_line(&mut self, line: [(Address, MemoryValue); LINE_SIZE]) {
        for (addr, value) in line.iter() {
            self.store_ubyte(*addr, *value);
        }
    }

    // pub fn show(&self) {
    //     for i in 0..MEMORY_SIZE {
    //         print!("{} {}\n", i, self.values[i]);
    //     }
    //     println!();
    // }

    // pub fn show_word(&self, addr: Address) {
    //     for i in 0..4 {
    //         print!("{}", self.get_byte(addr + 3 - i));
    //     }
    //     println!();
    // }
}
