default:
	ulimit -s unlimited && cargo run --release -- --bin main.bin --inst-stats --pc-stats

minrt_128:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 49167 --bin ./minrt_128.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./minrt_128.result

minrt_256:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 196623 --bin ./minrt_256.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./minrt_256.result

minrt_512:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 786432 --bin ./minrt_512.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./minrt_512.result

help:
	ulimit -s unlimited && cargo run --release -- --help

clean:
	cargo clean
