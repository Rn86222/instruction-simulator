mod cache;
mod core;
mod decoder;
mod fpu_emulator;
mod instruction;
mod instruction_memory;
mod label_map_loader;
mod memory;
mod register;
mod sld_loader;
mod types;
mod utils;
use crate::core::*;
use clap::Parser;

/// Simulator for CPUEX-Group2 computer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the input binary file.
    #[arg(short, long, default_value = "main.bin")]
    bin: String,

    /// Name of sld file for raytracing.
    #[arg(short, long, default_value = "./sld/contest.sld")]
    sld: String,

    /// Name of the output ppm file.
    #[arg(long)]
    ppm: Option<String>,

    /// No cache mode.
    /// If this flag is set, the simulator won't use cache.
    #[arg(short, long)]
    no_cache: bool,

    /// Take instruction statistics.
    #[arg(short, long)]
    inst_stats: bool,

    /// Take program-counter statistics.
    #[arg(long)]
    pc_stats: bool,

    /// Show output.
    #[arg(short, long)]
    show_output: bool,

    /// Show progress bar.
    /// If this flag is set with a value, the simulator will show progress bar.
    /// The value of this flag is the total size of output ppm file.
    #[arg(short, long, default_value = "0")]
    progress_bar_size: u64,

    /// Profiling mode.
    /// If this flag is set with a file name, the simulator will output framegraph with the given file name.
    #[arg(long)]
    prof: Option<String>,

    /// Label map file.
    /// If this flag is set with a file name, the simulator will use the given file as label map, and output pc graph.
    #[arg(long)]
    label_map: Option<String>,
}

fn main() {
    let mut core = Core::new();

    let args = Args::parse();
    let use_cache = !args.no_cache;
    let take_inst_stats = args.inst_stats;
    let take_pc_stats = args.pc_stats;
    let show_output = args.show_output;
    let progress_bar_size = args.progress_bar_size;
    let bin_file_path = args.bin.clone();
    let ppm_file_path = args.ppm.unwrap_or(args.bin.replace(".bin", ".ppm"));
    let sld_file_path = args.sld;
    let prof_file_path = args.prof;
    let label_map_file_path = args.label_map;
    let pc_graph_file_path = args.bin.replace(".bin", ".pc.dot");
    let props = CoreProps {
        use_cache,
        take_inst_stats,
        take_pc_stats,
        show_output,
        progress_bar_size,
        bin_file_path,
        ppm_file_path,
        sld_file_path,
        prof_file_path,
        label_map_file_path,
        pc_graph_file_path,
    };
    core.run(props);
}
