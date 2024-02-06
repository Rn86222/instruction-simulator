run:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 786447 --no-cache --bin minrt.bin --sld ./sld/contest.sld

full:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 196623 --inst-stats --label-map minrt.lmap --bin minrt.bin --sld ./sld/contest.sld

minrt:
	ulimit -s unlimited && cargo run --release -- --bin minrt.bin > minrt-result.txt

minrt_mini:
	ulimit -s unlimited && cargo run --release -- --bin minrt_mini.bin > minrt_mini-result.txt

minrt_256:
	ulimit -s unlimited && cargo run --release -- --bin minrt_256.bin > minrt_256-result.txt

help:
	ulimit -s unlimited && cargo run --release -- --help

clean:
	cargo clean
