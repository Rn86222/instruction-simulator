run:
	ulimit -s unlimited && cargo run --release -- --bin main.bin --verbose 2 > result.txt

minrt:
	ulimit -s unlimited && cargo run --release -- --bin minrt.bin > minrt-result.txt

minrt_mini:
	ulimit -s unlimited && cargo run --release -- --bin minrt_mini.bin > minrt_mini-result.txt

minrt_256:
	ulimit -s unlimited && cargo run --release -- --bin minrt_256.bin > minrt_256-result.txt

help:
	cargo run --release -- --help

clean:
	cargo clean
