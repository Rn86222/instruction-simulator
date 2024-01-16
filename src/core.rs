use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use crate::cache::*;
use crate::decoder::*;
use crate::fpu_emulator::*;
use crate::instruction::*;
use crate::instruction_memory::*;
use crate::memory::*;
use crate::register::*;
use crate::sld_loader::*;
use crate::types::*;
use crate::utils::*;

const INT_REGISTER_SIZE: usize = 32;
const FLOAT_REGISTER_SIZE: usize = 32;
const IO_ADDRESS: Address = 2147483648;

pub struct Core {
    memory: Memory,
    cache: Cache,
    memory_access_count: usize,
    cache_hit_count: usize,
    instruction_memory: InstructionMemory,
    instruction_count: InstructionCount,
    int_registers: [IntRegister; INT_REGISTER_SIZE],
    float_registers: [FloatRegister; FLOAT_REGISTER_SIZE],
    pc: Address,
    _pc_stats: HashMap<Address, (Instruction, usize)>,
    inst_stats: HashMap<String, usize>,
    inv_map: InvMap,
    sqrt_map: SqrtMap,
    sld_vec: Vec<String>,
    sld_counter: usize,
    output: Vec<u8>,
    decoded_instructions: Vec<Instruction>,
    use_cache: bool,
}

impl Core {
    pub fn new() -> Self {
        let memory = Memory::new();
        let cache = Cache::new();
        let memory_access_count = 0;
        let cache_hit_count = 0;
        let instruction_memory = InstructionMemory::new();
        let instruction_count = 0;
        let int_registers = [IntRegister::new(); INT_REGISTER_SIZE];
        let float_registers = [FloatRegister::new(); FLOAT_REGISTER_SIZE];
        let pc = 0;
        let _pc_stats = HashMap::new();
        let inst_stats = HashMap::new();
        let inv_map = create_inv_map();
        let sqrt_map = create_sqrt_map();
        let sld_vec = vec![];
        let sld_counter = 0;
        let output = vec![];
        let decoded_instructions = vec![];
        let use_cache = true;
        Core {
            memory,
            cache,
            memory_access_count,
            cache_hit_count,
            instruction_memory,
            instruction_count,
            int_registers,
            float_registers,
            pc,
            _pc_stats,
            inst_stats,
            inv_map,
            sqrt_map,
            sld_vec,
            sld_counter,
            output,
            decoded_instructions,
            use_cache,
        }
    }

    pub fn get_inv_map(&self) -> &InvMap {
        &self.inv_map
    }

    pub fn get_sqrt_map(&self) -> &SqrtMap {
        &self.sqrt_map
    }

    pub fn get_pc(&self) -> Address {
        self.pc
    }

    pub fn increment_pc(&mut self) {
        self.pc += 4;
    }

    pub fn set_pc(&mut self, new_pc: Address) {
        self.pc = new_pc;
    }

    fn increment_instruction_count(&mut self) {
        self.instruction_count += 1;
    }

    pub fn load_instruction(&mut self, addr: Address) -> InstructionValue {
        self.instruction_memory.load(addr)
    }

    pub fn store_instruction(&mut self, addr: Address, inst: InstructionValue) {
        self.instruction_memory.store(addr, inst);
    }

    pub fn get_int_register(&self, index: usize) -> Int {
        self.int_registers[index].get()
    }

    pub fn set_int_register(&mut self, index: usize, value: Int) {
        if index == ZERO {
            return; // zero register
        }
        self.int_registers[index].set(value);
    }

    pub fn get_float_register(&self, index: usize) -> FloatingPoint {
        self.float_registers[index].get()
    }

    pub fn set_float_register(&mut self, index: usize, value: FloatingPoint) {
        self.float_registers[index].set(value);
    }

    fn increment_memory_access_count(&mut self) {
        self.memory_access_count += 1;
    }

    fn increment_cache_hit_count(&mut self) {
        self.cache_hit_count += 1;
    }

    fn process_cache_miss(&mut self, addr: Address) {
        let line_addr = addr & !((1 << self.cache.get_offset_bit_num()) - 1);
        let line = self.memory.get_cache_line(line_addr);
        let set_line_result = self.cache.set_line(line_addr, line);
        if let Some(evicted_line) = set_line_result {
            self.memory.set_cache_line(evicted_line);
        }
    }

    #[allow(dead_code)]
    pub fn load_byte(&mut self, addr: Address) -> Byte {
        self.increment_memory_access_count();
        let cache_access = self.cache.get_ubyte(addr);
        match cache_access {
            CacheAccess::HitUByte(value) => {
                self.increment_cache_hit_count();
                u8_to_i8(value) as Byte
            }
            CacheAccess::Miss => {
                let value = self.memory.load_byte(addr);
                self.process_cache_miss(addr);
                value
            }
            _ => {
                panic!("invalid cache access");
            }
        }
    }

    #[allow(dead_code)]
    pub fn load_ubyte(&mut self, addr: Address) -> UByte {
        self.increment_memory_access_count();
        let cache_access = self.cache.get_ubyte(addr);
        match cache_access {
            CacheAccess::HitUByte(value) => {
                self.increment_cache_hit_count();
                value
            }
            CacheAccess::Miss => {
                let value = self.memory.load_ubyte(addr);
                self.process_cache_miss(addr);
                value
            }
            _ => {
                panic!("invalid cache access");
            }
        }
    }

    #[allow(dead_code)]
    pub fn store_byte(&mut self, addr: Address, value: Byte) {
        self.increment_memory_access_count();
        let cache_access = self.cache.set_ubyte(addr, i8_to_u8(value));
        match cache_access {
            CacheAccess::HitSet => {
                self.increment_cache_hit_count();
            }
            CacheAccess::Miss => {
                self.memory.store_byte(addr, value);
                self.process_cache_miss(addr);
            }
            _ => {
                panic!("invalid cache access");
            }
        }
    }

    #[allow(dead_code)]
    pub fn load_half(&mut self, addr: Address) -> Half {
        self.increment_memory_access_count();
        let cache_access = self.cache.get_uhalf(addr);
        match cache_access {
            CacheAccess::HitUHalf(value) => {
                self.increment_cache_hit_count();
                u16_to_i16(value)
            }
            CacheAccess::Miss => {
                let value = self.memory.load_half(addr);
                self.process_cache_miss(addr);
                value
            }
            _ => {
                panic!("invalid cache access");
            }
        }
    }

    #[allow(dead_code)]
    pub fn load_uhalf(&mut self, addr: Address) -> UHalf {
        self.increment_memory_access_count();
        let cache_access = self.cache.get_uhalf(addr);
        match cache_access {
            CacheAccess::HitUHalf(value) => {
                self.increment_cache_hit_count();
                value
            }
            CacheAccess::Miss => {
                let value = self.memory.load_uhalf(addr);
                self.process_cache_miss(addr);
                value
            }
            _ => {
                panic!("invalid cache access");
            }
        }
    }

    pub fn load_word(&mut self, addr: Address) -> Word {
        if addr == IO_ADDRESS {
            let value = self.sld_vec[self.sld_counter].parse::<i32>().unwrap();
            self.sld_counter += 1;
            return value;
        }
        if self.use_cache {
            let cache_access = self.cache.get_word(addr);
            match cache_access {
                CacheAccess::HitWord(value) => {
                    self.increment_cache_hit_count();
                    value
                }
                CacheAccess::Miss => {
                    let value = self.memory.load_word(addr);
                    self.process_cache_miss(addr);
                    value
                }
                _ => {
                    panic!("invalid cache access");
                }
            }
        } else {
            self.increment_memory_access_count();
            self.memory.load_word(addr)
        }
    }

    pub fn load_word_fp(&mut self, addr: Address) -> Word {
        if addr == IO_ADDRESS {
            let value = self.sld_vec[self.sld_counter].parse::<f32>().unwrap();
            let fp = FloatingPoint::new_f32(value);
            self.sld_counter += 1;
            u32_to_i32(fp.get_32_bits())
        } else {
            self.load_word(addr)
        }
    }

    #[allow(dead_code)]
    pub fn store_half(&mut self, addr: Address, value: Half) {
        self.increment_memory_access_count();
        let cache_access = self.cache.set_uhalf(addr, i16_to_u16(value));
        match cache_access {
            CacheAccess::HitSet => {
                self.increment_cache_hit_count();
            }
            CacheAccess::Miss => {
                self.memory.store_half(addr, value);
                self.process_cache_miss(addr);
            }
            _ => {
                panic!("invalid cache access");
            }
        }
    }

    pub fn store_word(&mut self, addr: Address, value: Word) {
        if addr == IO_ADDRESS {
            self.output.push(value as u8);
            return;
        }
        if self.use_cache {
            let cache_access = self.cache.set_word(addr, value);
            match cache_access {
                CacheAccess::HitSet => {
                    self.increment_cache_hit_count();
                }
                CacheAccess::Miss => {
                    self.memory.store_word(addr, value);
                    self.process_cache_miss(addr);
                }
                _ => {
                    panic!("invalid cache access");
                }
            }
        } else {
            self.memory.store_word(addr, value);
            self.increment_memory_access_count();
        }
    }

    #[allow(dead_code)]
    pub fn show_registers(&self) {
        for i in 0..INT_REGISTER_SIZE {
            print!("x{: <2} 0x{:>08x} ", i, self.get_int_register(i));
            if i % 8 == 7 {
                println!();
            }
        }
        for i in 0..FLOAT_REGISTER_SIZE {
            print!(
                "f{: <2} 0x{:>08x} ",
                i,
                self.get_float_register(i).get_32_bits()
            );
            if i % 8 == 7 {
                println!();
            }
        }
    }

    // #[allow(dead_code)]
    // fn update_pc_stats(&mut self) {
    //     if let Some(inst) = self.fetched_instruction {
    //         let decoded = decode_instruction(inst);
    //         if let Instruction::Other = decoded {
    //             return;
    //         }
    //         let pc = self.get_pc();
    //         self.pc_stats
    //             .entry(pc)
    //             .and_modify(|e| e.1 += 1)
    //             .or_insert((decoded, 1));
    //     }
    // }

    // fn show_pc_stats(&self) {
    //     println!("---------- pc stats ----------");
    //     let mut pc_stats = vec![];
    //     for (pc, (decoded, inst_count)) in &self.pc_stats {
    //         let inst = create_instruction_struct(*decoded);
    //         let inst_name = get_name(&inst);
    //         pc_stats.push((pc, inst_name, inst_count));
    //     }
    //     pc_stats.sort_by(|a, b| b.2.cmp(a.2));
    //     for pc_stat in &pc_stats {
    //         let pc_inst_string = format!("{:>08}({})", pc_stat.0, pc_stat.1);
    //         print_filled_with_space(&pc_inst_string, 25);
    //         println!("{}", pc_stat.2);
    //     }
    // }

    pub fn update_inst_stats(&mut self, inst_name: &str) {
        self.inst_stats
            .entry(inst_name.to_string())
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }

    fn show_inst_stats(&self) {
        println!("---------- inst stats ----------");
        let mut inst_stats = vec![];
        for (inst_name, inst_count) in &self.inst_stats {
            inst_stats.push((inst_name, inst_count));
        }
        inst_stats.sort_by(|a, b| b.1.cmp(a.1));
        for inst_stat in &inst_stats {
            print_filled_with_space(&inst_stat.0.to_string(), 8);
            println!(" {}", inst_stat.1);
        }
    }

    fn show_memory_stats(&self) {
        println!("memory access count: {}", self.memory_access_count);
        println!("cache hit count: {}", self.cache_hit_count);
        println!(
            "cache hit rate: {:.5}%",
            self.cache_hit_count as f64 / self.memory_access_count as f64 * 100.0
        );
    }

    fn show_output_result(&self) {
        println!("---------- output ----------");
        for i in 0..self.output.len() {
            println!(
                "{} {} 0x{:>02x} {}",
                i, self.output[i], self.output[i], self.output[i] as char
            );
        }
    }

    fn load_sld_file(&mut self, path: &str) {
        self.sld_vec = load_sld_file(path);
    }

    pub fn end(&mut self) {
        self.pc = INSTRUCTION_MEMORY_SIZE as Address;
    }

    fn decode_all_instructions(&mut self) {
        for i in 0..INSTRUCTION_MEMORY_SIZE {
            let inst = self.load_instruction(4 * i as Address);
            let decoded = decode_instruction(inst);
            self.decoded_instructions.push(decoded);
        }
    }

    pub fn run(
        &mut self,
        _verbose: u32,
        use_cache: bool,
        ppm_file_path: &str,
        sld_file_path: &str,
    ) {
        let start_time = Instant::now();
        let mut cycle_num: u128 = 0;

        let mut ppm_file = File::create(ppm_file_path).unwrap();
        let mut before_output_len = 0;

        self.load_sld_file(sld_file_path);
        self.decode_all_instructions();

        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(1000)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .unwrap();

        self.use_cache = use_cache;

        loop {
            cycle_num += 1;
            if self.get_pc() >= INSTRUCTION_MEMORY_SIZE as Address {
                println!("End of program.");
                break;
            }

            let instrucion = self.decoded_instructions[self.get_pc() as usize >> 2];
            exec_instruction(instrucion, self);

            // self.update_inst_stats();
            // self.update_pc_stats();

            if cycle_num % 10000000 == 0 {
                eprint!(
                    "\r{} {:>08} pc: {:>06} sp: {:>010}",
                    self.instruction_count,
                    self.output.len(),
                    self.get_pc(),
                    self.get_int_register(2),
                );
            }

            self.increment_instruction_count();

            if before_output_len != self.output.len() {
                for i in before_output_len..self.output.len() {
                    let byte = [self.output[i]];
                    ppm_file.write_all(&byte).unwrap();
                }
                before_output_len = self.output.len();
            }
        }

        if let Ok(report) = guard.report().build() {
            let file = File::create("flamegraph_256_inline.svg").unwrap();
            report.flamegraph(file).unwrap();
        };

        println!(
            "executed instruction count: {}\nelapsed time: {:?}\n{:.2} MIPS",
            self.instruction_count,
            start_time.elapsed(),
            self.instruction_count as f64 / start_time.elapsed().as_micros() as f64
        );
        self.show_memory_stats();
        self.show_output_result();
        self.show_inst_stats();
        // self.show_pc_stats();
    }
}
