#!/bin/bash

TXT_RED="\e[31m"
TXT_YELLOW="\e[33m"
TXT_GREEN="\e[32m"
TXT_BLUE="\e[34m"
TXT_BRIGHT_YELLOW="\e[93m"
TXT_BRIGHT_CYAN="\e[96m"
TXT_BRIGHT_MAGENTA="\e[95m"
TXT_BRIGHT_GREEN="\e[92m"
TXT_CLEAR="\e[0m"

function println_yellow {
    printf "${TXT_YELLOW}${1}${TXT_CLEAR}\n"
}
function println_cyan {
    printf "${TXT_BRIGHT_CYAN}${1}${TXT_CLEAR}\n"
}
function println_red {
    printf "${TXT_RED}${1}${TXT_CLEAR}\n"
}
function println_green {
    printf "${TXT_GREEN}${1}${TXT_CLEAR}\n"
}
function println_blue {
    printf "${TXT_BLUE}${1}${TXT_CLEAR}\n"
}
function println_magenta {
    printf "${TXT_BRIGHT_MAGENTA}${1}${TXT_CLEAR}\n"
}



println_yellow "Building in release mode\n"

cargo build -r

println_blue "Installing latest version of fastpasta from crates.io\n"

cargo install fastpasta --locked

println_blue "Installing hyperfine for benchmarking"

cargo install hyperfine --locked


println_cyan "\nChecking version of local fastpasta build"

target/release/fastpasta --version

println_cyan "\nChecking version of remote fastpasta installation"

fastpasta --version

println_magenta "\n===================================================================================================== "
println_magenta "*********************************************************************************"
println_magenta "***                                                                           ***"
println_magenta "*** Benchmarking the local compiled binary vs. the latest remote installation ***"
println_magenta "***                                                                           ***"
println_magenta "*********************************************************************************\n"

tests_files_array=()

# Stores output of each test, from which the benchmark result is extracted and evaluated.
bench_results_file="bench_comp_tdh_no_data_ihw.md"
# Regex to extract the mean timings for each tested version of fastpasta
re_mean_timings="(?<=\` \| )\d*(?=\.)"
# Stores the mean timings of the local fastpasta vs. the remote (negative values -> the local is faster)
bench_results_local_mean_diff=()


tdh_no_data_ihw="tests/test-data/tdh_no_data_ihw.raw"
println_magenta "Benchmarking file ${tdh_no_data_ihw} with command: ${cmd}"

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

local_test=local__fastpasta__check_all_its_stave

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

bench_check_all_its_stave "${tdh_no_data_ihw}"
local_mean=${mean_timings[0]}
remote_mean=${mean_timings[1]}

println_yellow "Local fastpasta timing (mean): ${local_mean} ms"
println_yellow "Remote fastpasta timing (mean): ${remote_mean} ms"

local_minus_remote=$((${local_mean}-${remote_mean}))
remote_minus_local=$((${remote_mean}-${local_mean}))

bench_results_local_mean_diff+=(${local_minus_remote})
println_yellow "bench_results_local_mean_diff: ${bench_results_local_mean_diff[@]}"

if [[ "${local_mean}" -lt ${remote_mean} ]]; then
    println_green "local build is faster by ${remote_minus_local} ms!"
elif [[ "${local_mean}" -gt ${remote_mean} ]]; then
    println_red "local build is slower by ${local_minus_remote} ms..."
elif [[ "${local_mean}" -eq ${remote_mean} ]]; then
    println_blue "local and remote are about equally fast"
fi



total_diff=0
for i in ${bench_results_local_mean_diff[@]}; do
    let total_diff+=$i
done

total_test_count=${#bench_results_local_mean_diff[@]}

println_magenta "Total timing difference in ${total_test_count} tests: ${total_diff} ms"


avg_diff=$(awk -v sum=$total_diff -v total_tests=${total_test_count} 'BEGIN { print sum/total_tests }')

println_magenta "Average timing difference: ${avg_diff} ms"

if (( ${avg_diff} == 0  )); then
    println_blue "No difference"
elif (( ${avg_diff} < 0 )); then
    println_green "Nice! Seems faster overall!"
elif (( ${avg_diff} > 10 )); then
        println_red "This is really bad... D:"
else
    println_red  "It's slower but it could be worse..."
fi

rm ${bench_results_file}
