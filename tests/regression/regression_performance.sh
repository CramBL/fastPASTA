#!/bin/bash

source ./tests/regression/utils.sh

println_yellow "Building in release mode\n"

cargo build -r

println_blue "Installing latest version of fastpasta from crates.io\n"

cargo install fastpasta --locked

println_blue "Installing hyperfine for benchmarking"

cargo install hyperfine --locked


println_cyan "\nChecking version of local fastpasta build"

target/release/fastpasta --version

println_cyan "Checking version of remote fastpasta installation"

fastpasta --version

println_magenta "\n===================================================================================================== "
println_magenta "*********************************************************************************"
println_magenta "***                                                                           ***"
println_magenta "*** Benchmarking the local compiled binary vs. the latest remote installation ***"
println_magenta "***                                                                           ***"
println_magenta "*********************************************************************************\n"

file_path="tests/test-data/"

# Files used in benchmarks
file_tdh_no_data_ihw="tdh_no_data_ihw.raw"
file_10_rdh="10_rdh.raw"
file_readout_superpage1="readout.superpage.1.raw"
file_rawtf_epn180_l6_1="rawtf_epn180_l6_1.raw"

tests_files_array=(
    file_10_rdh file_readout_superpage1 file_tdh_no_data_ihw file_rawtf_epn180_l6_1
)

# Stores output of each test, from which the benchmark result is extracted and evaluated.
bench_results_file="bench_comp.md"
# Regex to extract the mean timings for each tested version of fastpasta
re_mean_timings="(?<=\` \| )\d*(?=\.)"
# Stores the mean timings of the local fastpasta vs. the remote (negative values -> the local is faster)
bench_results_local_mean_diff=()



cmd="check all its-stave"
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

declare -a mean_timings
function bench_check_all_its_stave {
    input_file=$1
    hyperfine --style full\
        --warmup 10\
        --time-unit millisecond\
        --parameter-list fastpasta local__fastpasta__check_all_its_stave,remote__fastpasta__check_all_its_stave\
        --shell=bash "{fastpasta} ${input_file}" --export-markdown ${bench_results_file}

    timing_res=( $(cat ${bench_results_file} | grep -Po "${re_mean_timings}" | head -n 2) )
    mean_timings[0]=${timing_res[0]}
    mean_timings[1]=${timing_res[1]}
}

for file in "${tests_files_array[@]}"; do
    # The file is a value, to get the contents we need to use `declare -n`
    declare -n test_file=$file
    println_magenta "\n ==> Benchmarking file ${file_path}${test_file} with command: ${cmd}\n"

    bench_check_all_its_stave "${file_path}${test_file}"
    local_mean=${mean_timings[0]}
    remote_mean=${mean_timings[1]}

    println_yellow "\n\tLocal fastpasta timing : ${local_mean} ms (mean)"
    println_yellow "\n\tRemote fastpasta timing: ${remote_mean} ms (mean)"

    local_minus_remote=$((${local_mean}-${remote_mean}))
    remote_minus_local=$((${remote_mean}-${local_mean}))

    bench_results_local_mean_diff+=(${local_minus_remote})
    println_bright_yellow "\t\tdifference between local and remote build: ${bench_results_local_mean_diff[@]} ms"

    if [[ "${local_mean}" -lt ${remote_mean} ]]; then
        println_green "\t\t-> local build is faster by ${remote_minus_local} ms!"
    elif [[ "${local_mean}" -gt ${remote_mean} ]]; then
        println_red "\t\t-> local build is slower by ${local_minus_remote} ms..."
    elif [[ "${local_mean}" -eq ${remote_mean} ]]; then
        println_blue "\t\t-> local and remote are about equally fast"
    fi

done


println_magenta "*********************************************************************************"
println_magenta "***                  SUMMARY OF PERFORMANCE REGRESSION TESTS                  ***"
println_magenta "*********************************************************************************\n"

total_diff=0
for i in ${bench_results_local_mean_diff[@]}; do
    let total_diff+=$i
done

total_test_count=${#bench_results_local_mean_diff[@]}
println_magenta "Total timing difference in ${total_test_count} tests: ${total_diff} ms"

avg_diff=$(awk -v sum=$total_diff -v total_tests=${total_test_count} 'BEGIN { print sum/total_tests }')
println_magenta "Average timing difference: ${avg_diff} ms"

println_cyan "\n--- RESULT --- \n"

if [[ $(float_cmp ${avg_diff} 0) == 0 ]]; then
    println_blue "No difference"
elif [[ $(float_cmp ${avg_diff} 0) -eq 2 ]]; then
    println_green "Nice! Seems faster overall!"
elif [[ $(float_cmp ${avg_diff} 10) -eq 1 ]]; then
    println_red "This is really bad... D:"
else
    println_bright_yellow  "It's slower but it could be worse..."
fi

rm ${bench_results_file}
