use indicatif::{ProgressBar, ProgressStyle};
// use std::collections::HashMap;
use fxhash::FxHashMap;
use petgraph::dot::Dot;
use petgraph::Graph;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::time::Instant;
use std::vec;

// use crate::cache::*;
use crate::decoder::*;
use crate::fpu_emulator::*;
use crate::instruction::*;
use crate::instruction_memory::*;
use crate::label_map_loader::*;
use crate::memory::*;
use crate::pseudo_lru_cache::*;
use crate::register::*;
use crate::sld_loader::*;
use crate::types::*;
use crate::utils::*;

const INT_REGISTER_SIZE: usize = 64;
const FLOAT_REGISTER_SIZE: usize = 64;
// const IO_ADDRESS: Address = 2147483648;

const CACHE_MISS_STALL: usize = 60;
const CACHE_HIT_STALL: usize = 1;
const FLUSH_STALL: usize = 3;
#[allow(dead_code)]
const FREQUENCY: usize = 65 * 1000000;
#[allow(dead_code)]
const BAUD_RATE: usize = 11520;

pub struct Core {
    memory: Memory,
    cache: PseudoLRUCache,
    memory_access_count: usize,
    cache_hit_count: usize,
    cache_miss_count: usize,
    instruction_memory: InstructionMemory,
    instruction_count: InstructionCount,
    int_registers: [IntRegister; INT_REGISTER_SIZE],
    float_registers: [FloatRegister; FLOAT_REGISTER_SIZE],
    pc: Address,
    pc_stats: [(usize, InstructionId); 1000000],
    pc_graph: Graph<String, usize>,
    pc_to_node_id_map: FxHashMap<Address, petgraph::graph::NodeIndex>,
    current_pc_node_id: petgraph::graph::NodeIndex,
    create_pc_graph: bool,
    inst_stats: [usize; 256],
    int_registers_access_counter: Vec<usize>,
    float_registers_access_counter: Vec<usize>,
    inv_map: InvMap,
    sqrt_map: SqrtMap,
    sld_vec: Vec<String>,
    sld_counter: usize,
    output: Vec<u8>,
    decoded_instructions: Vec<Instruction>,
    use_cache: bool,
    load_stall_counter: usize,
    load_dest: Option<usize>,
    before_load_dest: Option<usize>,
    fpu_stall_counter: usize,
    flush_counter: usize,
}

impl Core {
    pub fn new() -> Self {
        let memory = Memory::new();
        let cache = PseudoLRUCache::new();
        let memory_access_count = 0;
        let cache_hit_count = 0;
        let cache_miss_count = 0;
        let instruction_memory = InstructionMemory::new();
        let instruction_count = 0;
        let int_registers = [IntRegister::new(); INT_REGISTER_SIZE];
        let float_registers = [FloatRegister::new(); FLOAT_REGISTER_SIZE];
        let pc = 0;
        let pc_stats = [(0, InstructionId::End); 1000000];
        let mut pc_graph = Graph::new();
        let pc_to_node_id_map = FxHashMap::default();
        let current_node_id = pc_graph.add_node("START".to_string());
        let create_pc_graph = false;
        let inst_stats = [0; 256];
        let int_registers_access_counter = vec![0; INT_REGISTER_SIZE];
        let float_registers_access_counter = vec![0; FLOAT_REGISTER_SIZE];
        let inv_map = create_inv_map();
        let sqrt_map = create_sqrt_map();
        let sld_vec = vec![];
        let sld_counter = 0;
        let output = vec![];
        let decoded_instructions = vec![];
        let use_cache = true;
        let load_stall_counter = 0;
        let load_dest = None;
        let before_load_dest = None;
        let fpu_stall_counter = 0;
        let flush_counter = 0;
        Core {
            memory,
            cache,
            memory_access_count,
            cache_hit_count,
            cache_miss_count,
            instruction_memory,
            instruction_count,
            int_registers,
            float_registers,
            pc,
            pc_stats,
            pc_graph,
            pc_to_node_id_map,
            current_pc_node_id: current_node_id,
            create_pc_graph,
            inst_stats,
            int_registers_access_counter,
            float_registers_access_counter,
            inv_map,
            sqrt_map,
            sld_vec,
            sld_counter,
            output,
            decoded_instructions,
            use_cache,
            load_stall_counter,
            load_dest,
            before_load_dest,
            fpu_stall_counter,
            flush_counter,
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
        if self.create_pc_graph {
            self.update_pc_graph();
        }
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

    pub fn get_int_register(&mut self, index: usize) -> Int {
        self.int_registers_access_counter[index] += 1;
        if self.before_load_dest == Some(index) {
            self.load_stall_counter += 1;
        }
        self.int_registers[index].get()
    }

    pub fn set_int_register(&mut self, index: usize, value: Int) {
        self.int_registers_access_counter[index] += 1;
        if index == ZERO {
            return; // zero register
        }
        self.int_registers[index].set(value);
    }

    pub fn get_float_register(&mut self, index: usize) -> FloatingPoint {
        self.float_registers_access_counter[index] += 1;
        if index == ZERO {
            return FloatingPoint::new(0); // zero register
        }
        if self.before_load_dest == Some(index + 32) {
            self.load_stall_counter += 1;
        }
        self.float_registers[index].get()
    }

    pub fn set_float_register(&mut self, index: usize, value: FloatingPoint) {
        self.float_registers_access_counter[index] += 1;
        self.float_registers[index].set(value);
    }

    fn increment_memory_access_count(&mut self) {
        self.memory_access_count += 1;
    }

    fn increment_cache_hit_count(&mut self) {
        self.cache_hit_count += 1;
    }

    pub fn increment_fpu_stall_counter(&mut self, value: usize) {
        self.fpu_stall_counter += value;
    }

    fn show_fpu_stall_counter(&self) {
        println!("fpu stall: {}", self.fpu_stall_counter);
    }

    pub fn set_load_dest(&mut self, value: usize) {
        self.load_dest = Some(value);
    }

    fn show_load_stall_counter(&self) {
        println!("load stall: {}", self.load_stall_counter);
    }

    pub fn increment_flush_counter(&mut self) {
        self.flush_counter += 1;
    }

    fn increment_cache_miss_count(&mut self) {
        self.cache_miss_count += 1;
    }

    fn process_cache_miss(&mut self, addr: Address) {
        let line_addr = addr & !((1 << self.cache.get_offset_bit_num()) - 1);
        let line = self.memory.get_cache_line(line_addr);
        let set_line_result = self.cache.set_line(line_addr, line);
        if let Some(evicted_line) = set_line_result {
            self.memory.set_cache_line(evicted_line);
        }
    }

    pub fn read_int(&mut self) -> Word {
        let value = self.sld_vec[self.sld_counter].parse::<i32>().unwrap();
        self.sld_counter += 1;
        value
    }

    pub fn read_float(&mut self) -> Word {
        let value = self.sld_vec[self.sld_counter].parse::<f32>().unwrap();
        let fp = FloatingPoint::new_f32(value);
        self.sld_counter += 1;
        u32_to_i32(fp.get_32_bits())
    }

    pub fn load_word(&mut self, addr: Address) -> Word {
        self.increment_memory_access_count();
        if self.use_cache {
            let cache_access = self.cache.get_word(addr);
            match cache_access {
                CacheAccess::HitWord(value) => {
                    self.increment_cache_hit_count();
                    value
                }
                CacheAccess::Miss => {
                    self.increment_cache_miss_count();
                    let value = self.memory.load_word(addr);
                    self.process_cache_miss(addr);
                    value
                }
                _ => {
                    panic!("invalid cache access");
                }
            }
        } else {
            self.memory.load_word(addr)
        }
    }

    pub fn print_char(&mut self, value: Word) {
        self.output.push(value as u8);
    }

    pub fn print_int(&mut self, value: Word) {
        self.output
            .append(&mut value.to_string().as_bytes().to_vec());
    }

    pub fn store_word(&mut self, addr: Address, value: Word) {
        self.increment_memory_access_count();
        if self.use_cache {
            let cache_access = self.cache.set_word(addr, value);
            match cache_access {
                CacheAccess::HitSet => {
                    self.increment_cache_hit_count();
                }
                CacheAccess::Miss => {
                    self.increment_cache_miss_count();
                    self.memory.store_word(addr, value);
                    self.process_cache_miss(addr);
                }
                _ => {
                    panic!("invalid cache access");
                }
            }
        } else {
            self.memory.store_word(addr, value);
        }
    }

    #[allow(dead_code)]
    pub fn show_registers(&self) {
        for i in 0..INT_REGISTER_SIZE {
            print!("x{: <2} 0x{:>08x} ", i, self.int_registers[i].get());
            if i % 8 == 7 {
                println!();
            }
        }
        for i in 0..FLOAT_REGISTER_SIZE {
            print!(
                "f{: <2} 0x{:>08x} ",
                i,
                self.float_registers[i].get().get_32_bits()
            );
            if i % 8 == 7 {
                println!();
            }
        }
    }

    pub fn update_inst_stats(&mut self, inst_id: InstructionId) {
        self.inst_stats[inst_id as usize] += 1;
    }

    pub fn update_pc_stats(&mut self, pc: u32, inst_id: InstructionId) {
        self.pc_stats[(pc >> 2) as usize].0 += 1;
        self.pc_stats[(pc >> 2) as usize].1 = inst_id;
    }

    fn show_inst_stats(&self) {
        println!("---------- inst stats ----------");
        // let inst_id_to_name_map = create_inst_id_to_name_map();
        let mut inst_stats = vec![];
        for (id, inst_count) in self.inst_stats.iter().enumerate() {
            if *inst_count == 0 {
                continue;
            }
            inst_stats.push((INST_ID_TO_NAME[id], inst_count));
        }
        inst_stats.sort_by(|a, b| b.1.cmp(a.1));
        for inst_stat in &inst_stats {
            print_filled_with_space(&inst_stat.0.to_string(), 8);
            println!(" {}", inst_stat.1);
        }
    }

    fn show_pc_stats(&self) {
        println!("---------- pc stats ----------");
        let mut pc_stats = vec![];
        for (pc, (count, inst_id)) in self.pc_stats.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            pc_stats.push((pc, count, INST_ID_TO_NAME[*inst_id as usize]));
        }
        pc_stats.sort_by(|a, b| b.1.cmp(a.1));
        for pc_stat in &pc_stats {
            let pc_inst_string = format!("{:>08}({})", pc_stat.0 * 4, pc_stat.2);
            print_filled_with_space(&pc_inst_string, 25);
            println!("{}", pc_stat.1);
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

    fn show_registers_access_counter(&self) {
        println!("---------- registers counter ----------");
        for i in 0..INT_REGISTER_SIZE {
            print!("x{: <2} {: >10}  ", i, self.int_registers_access_counter[i]);
            if i % 8 == 7 {
                println!();
            }
        }
        for i in 0..FLOAT_REGISTER_SIZE {
            print!(
                "f{: <2} {: >10}  ",
                i, self.float_registers_access_counter[i]
            );
            if i % 8 == 7 {
                println!();
            }
        }
    }

    fn output_pc_graph(&self, pc_graph_file_path: &str) {
        let mut f = std::fs::File::create(pc_graph_file_path).unwrap();
        let dot_str = format!("{:?}", Dot::with_config(&self.pc_graph, &[]));
        f.write_all(dot_str.as_bytes()).unwrap();
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

    fn load_bin_file(&mut self, bin_file: &str) {
        match File::open(bin_file) {
            Err(e) => {
                panic!("Failed in opening file ({}).", e);
            }
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();
                let mut inst_count = 0;
                let mut inst = 0;
                for byte in &buf {
                    inst += (*byte as u32) << ((inst_count % 4) * 8);
                    inst_count += 1;
                    if inst_count % 4 == 0 {
                        self.store_instruction(inst_count - 4, inst);
                        inst = 0;
                    }
                }
                if inst_count % 4 != 0 {
                    panic!("Reading file failed.\nThe size of sum of instructions is not a multiple of 4. {}", inst_count);
                }
            }
        }
    }

    fn load_label_map_file(&mut self, label_map_file: &str) {
        let label_map = load_label_map_file(label_map_file);
        for (addr, label) in label_map {
            let node_id = self.pc_graph.add_node(label.clone());
            self.pc_to_node_id_map.insert(addr, node_id);
        }
    }

    fn update_pc_graph(&mut self) {
        let new_node_id = self.pc_to_node_id_map.get(&self.get_pc());
        if let Some(node_id) = new_node_id {
            if let Some(edge_id) = self.pc_graph.find_edge(self.current_pc_node_id, *node_id) {
                let edge = self.pc_graph.edge_weight_mut(edge_id).unwrap();
                *edge += 1;
            } else {
                self.pc_graph.add_edge(self.current_pc_node_id, *node_id, 1);
            }
            self.current_pc_node_id = *node_id;
        }
    }

    fn init(&mut self) {
        self.set_int_register(RA, INSTRUCTION_MEMORY_SIZE as Int);
        self.set_int_register(SP, MEMORY_SIZE as Int);
    }

    fn show_progress(&self, progress_bar_size: u64, pb: &ProgressBar) {
        if progress_bar_size == 0 {
            eprint!(
                "\rinst count: {} output: {:>08} pc: {:>06} sp: {:>010}",
                self.instruction_count,
                self.output.len(),
                self.get_pc(),
                self.int_registers[SP].get(),
            );
        } else {
            pb.set_position(self.output.len() as u64);
        }
    }

    pub fn run(&mut self, props: CoreProps) {
        let start_time = Instant::now();
        let mut cycle_num: u128 = 0;

        let mut ppm_file = File::create(props.ppm_file_path).unwrap();
        let mut before_output_len = 0;

        self.init();
        self.load_bin_file(&props.bin_file_path);
        self.load_sld_file(&props.sld_file_path);
        if let Some(label_map_file) = &props.label_map_file_path {
            self.load_label_map_file(label_map_file);
            self.create_pc_graph = true;
        }
        self.decode_all_instructions();

        let pb = ProgressBar::new(props.progress_bar_size);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} \n {msg}")
        .unwrap()
        .progress_chars("#>-"));

        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(10000)
            .blocklist(&["libc", "libgcc", "pthread", "vdso"])
            .build()
            .unwrap();

        self.use_cache = props.use_cache;

        loop {
            self.before_load_dest = self.load_dest;
            self.load_dest = None;

            cycle_num += 1;
            if self.get_pc() >= INSTRUCTION_MEMORY_SIZE as Address {
                pb.finish_with_message("End of program.");
                break;
            }

            let instrucion = self.decoded_instructions[self.get_pc() as usize >> 2];
            let pc = self.get_pc();
            let inst_id = exec_instruction(instrucion, self);
            if props.take_inst_stats {
                self.update_inst_stats(inst_id);
            }
            if props.take_pc_stats {
                self.update_pc_stats(pc, inst_id);
            }
            if cycle_num % 10000000 == 0 {
                self.show_progress(props.progress_bar_size, &pb);
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

        if let Some(prof_file_path) = props.prof_file_path {
            if let Ok(report) = guard.report().build() {
                let file = File::create(prof_file_path).unwrap();
                report.flamegraph(file).unwrap();
            };
        }

        let cycle_num = self.instruction_count
            + self.flush_counter as u128 * FLUSH_STALL as u128
            + self.cache_miss_count as u128 * CACHE_MISS_STALL as u128
            + self.cache_hit_count as u128 * CACHE_HIT_STALL as u128
            + self.fpu_stall_counter as u128;
        let predicted_time = cycle_num as f64 * 1.59512149e-08
            + self.output.len() as f64 * 2.72652771e-05
            + 121.06771803862078;

        println!("flush count: {}", self.flush_counter);
        println!("cache miss count: {}", self.cache_miss_count);
        println!("predicted cycle count: {}", cycle_num);
        println!("predicted execution time: {:.2}s", predicted_time);

        println!(
            "executed instruction count: {}\nelapsed time: {:?}\n{:.2} MIPS",
            self.instruction_count,
            start_time.elapsed(),
            self.instruction_count as f64 / start_time.elapsed().as_micros() as f64
        );
        self.show_memory_stats();
        self.show_fpu_stall_counter();
        self.show_load_stall_counter();
        self.show_registers_access_counter();
        if props.take_inst_stats {
            self.show_inst_stats();
        }
        if props.take_pc_stats {
            self.show_pc_stats();
        }
        if props.show_output {
            self.show_output_result();
        }
        if self.create_pc_graph {
            self.output_pc_graph(&props.pc_graph_file_path);
        }
    }
}

pub struct CoreProps {
    pub take_inst_stats: bool,
    pub take_pc_stats: bool,
    pub use_cache: bool,
    pub show_output: bool,
    pub progress_bar_size: u64,
    pub bin_file_path: String,
    pub ppm_file_path: String,
    pub sld_file_path: String,
    pub prof_file_path: Option<String>,
    pub label_map_file_path: Option<String>,
    pub pc_graph_file_path: String,
}
