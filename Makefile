default:
	ulimit -s unlimited && cargo run --release -- --bin main.bin --inst-stats --pc-stats

minrt_64:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 12303 --bin ./test/minrt_64_u4_i150.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./test/minrt_64_u4_i150.result

minrt_128:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 49167 --bin ./test/minrt_128_u4_i150.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./test/minrt_128_u4_i150.result

minrt_256:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 196623 --bin ./test/minrt_256_u4_i150.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./test/minrt_256_u4_i150.result

minrt_512:
	ulimit -s unlimited && cargo run --release -- --progress-bar-size 786432 --bin ./minrt_512.bin --inst-stats --pc-stats --sld ./sld/contest.sld > ./minrt_512.result

help:
	ulimit -s unlimited && cargo run --release -- --help

clean:
	cargo clean
