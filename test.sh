#!/bin/bash -eu

minrt='minrt_256'
current_path=`pwd`
dirpath=__test_`cat /dev/urandom | tr -dc 'a-z0-9' | fold -w 16 | head -n 1`

function error() {
    status=$?
    cd $current_path
    rm -rf $dirpath
    echo "Exit status: $status"
    exit $status
}

trap 'error' 1 2 3 15

mkdir $dirpath
cd $dirpath

echo -n "Cloning compiler... "
git clone https://github.com/utokyo-compiler/cpuex-2-2023.git > /dev/null 2>&1
echo "done."
cd cpuex-2-2023
git checkout rn > /dev/null 2>&1
./to_riscv
set +e
make default > /dev/null 2>&1 &
rm test/$minrt.s > /dev/null 2>&1
cd ..

echo -n "Cloning assembler... "
git clone https://github.com/Rn86222/cpuex2-assembler.git > /dev/null 2>&1
echo "done."
cd cpuex2-assembler
set +e
rm ./$minrt.s ./$minrt.bin ./$minrt.data > /dev/null 2>&1
set -e
cargo build --release > /dev/null 2>&1 &
cd ..

echo -n "Cloning simulator... "
git clone https://github.com/Rn86222/instruction-simulator.git > /dev/null 2>&1
echo "done."
cd instruction-simulator
set +e
rm ./$minrt.bin ./$minrt.ppm > /dev/null 2>&1
set -e
cargo build --release > /dev/null 2>&1 &
cd ..

echo -n "Compiling '$minrt.ml'... "
cd cpuex-2-2023
set -e
if [ ! -e ./min-caml ]; then
    echo "Building compiler failed"
    exit 1
fi
./min-caml -inline 100 test/$minrt > /dev/null 2>&1
mv test/$minrt.s ../$minrt.s
cd ..
rm -rf cpuex-2-2023
echo "done."

echo -n "Assembling '$minrt.s'... "
cd cpuex2-assembler
mv ../$minrt.s ./$minrt.s
./target/release/assembler --file $minrt.s --style bin > /dev/null 2>&1
if [ ! -e ./$minrt.bin ]; then
    echo "Assembling Failed"
    exit 1
fi
mv ./$minrt.bin ../$minrt.bin
cd ..
rm -rf cpuex2-assembler
echo "done."

echo "Simulating '$minrt.bin'... "
cd instruction-simulator
mv ../$minrt.bin ./$minrt.bin
while [ ! -e ./target/release/instruction-simulator ]; do
    sleep 1
done
ulimit -s unlimited && ./target/release/instruction-simulator --no-cache --progress-bar-size 196623 --inst-stats --bin $minrt.bin --sld ./sld/contest.sld > result$dirpath.txt
echo "done."

echo "Checking '$minrt.ppm'..."
diff $minrt.ppm $minrt'_ans.ppm' > diff$dirpath.txt
if [ -s ./diff$dirpath.txt ]; then
    echo "Failed"
    mv diff$dirpath.txt ../../diff$dirpath.txt
else
    echo "Success"
fi

mv result$dirpath.txt ../../result$dirpath.txt
echo "Result saved to result$dirpath.txt"

cd ..
rm -rf instruction-simulator
cd ..
rm -rf $dirpath
echo "Done."
