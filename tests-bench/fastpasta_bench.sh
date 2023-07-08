#!/bin/bash

printf "Building in release mode\n\n"

cargo build -r

printf "Installing latest version of fastpasta from crates.io\n\n"

cargo install fastpasta --locked

printf "Installing hyperfine for benchmarking\n\n"

cargo install hyperfine --locked

quiet cargo install cowsay


printf "\nChecking version of local fastpasta build\n"

target/release/fastpasta --version

printf "\nChecking version of remote fastpasta installation\n"

fastpasta --version

printf "\n===================================================================================================== \n"
printf "*********************************************************************************\n"
printf "***                                                                           ***\n"
printf "*** Benchmarking the local compiled binary vs. the latest remote installation ***\n"
printf "***                                                                           ***\n"
printf "*********************************************************************************\n\n"

# Stores output of each test, from which the benchmark result is extracted and evaluated.
bench_results_file="bench_comp_tdh_no_data_ihw.md"
# Regex to extract the mean timings for each tested version of fastpasta
re_mean_timings="(?<=\` \| )\d*(?=\.)"
# Stores the mean timings of the local fastpasta vs. the remote (negative values -> the local is faster)
bench_results_local_mean_diff=()

cmd="check all its-stave"
tdh_no_data_ihw="tests/test-data/tdh_no_data_ihw.raw"
cowsay "Benchmarking file ${tdh_no_data_ihw} with command: ${cmd}"

function local__fastpasta__check_all_its_stave {
    file=$1
    target/release/fastpasta $file $cmd
}
export -f local__fastpasta__check_all_its_stave

function remote__fastpasta__check_all_its_stave {
    file=$1
    fastpasta $file $cmd
}
export -f remote__fastpasta__check_all_its_stave

hyperfine --style full\
        --warmup 10\
        --time-unit millisecond\
        --parameter-list fastpasta local__fastpasta__check_all_its_stave,remote__fastpasta__check_all_its_stave\
        --shell=bash "{fastpasta} ${tdh_no_data_ihw}" --export-markdown ${bench_results_file}


timings=( $(cat ${bench_results_file} | grep -Po "${re_mean_timings}" | head -n 2) )

local_mean=${timings[0]}
remote_mean=${timings[1]}
echo "Local fastpasta timing (mean): ${local_mean} ms"
echo "Remote fastpasta timing (mean): ${remote_mean} ms"

local_minus_remote=$((${local_mean}-${remote_mean}))
remote_minus_local=$((${remote_mean}-${local_mean}))

bench_results_local_mean_diff+=(${local_minus_remote})
echo "bench_results_local_mean_diff: ${bench_results_local_mean_diff[@]}"

if [[ "${local_mean}" -lt ${remote_mean} ]]; then
    cowsay "local build is faster by ${remote_minus_local} ms!"
elif [[ "${local_mean}" -gt ${remote_mean} ]]; then
    cowsay --dead "local build is slower by ${local_minus_remote} ms..."
elif [[ "${local_mean}" -eq ${remote_mean} ]]; then
    echo "local and remote are about equally fast"
fi



total_diff=0
for i in ${bench_results_local_mean_diff[@]}; do
    let total_diff+=$i
done

total_test_count=${#bench_results_local_mean_diff[@]}

echo "Total timing difference in ${total_test_count} tests: ${total_diff} ms"


avg_diff=$(awk -v sum=$total_diff -v total_tests=${total_test_count} 'BEGIN { print sum/total_tests }')

echo "Average timing difference: ${avg_diff} ms"

if (( ${avg_diff} == 0  )); then
    printf "No difference"
elif (( ${avg_diff} < 0 )); then
    cowsay "Nice! Seems faster overall!"
elif (( ${avg_diff} > 10 )); then
        cowsay --dead "This is really bad... D:"
else
    cowsay --dead "It's slower but it could be worse..."
fi

rm ${bench_results_file}
