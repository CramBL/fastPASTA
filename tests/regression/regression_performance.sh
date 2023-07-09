#!/bin/bash

# shellcheck disable=SC1091
source ./tests/regression/utils.sh

println_yellow "Building in release mode\n"

cargo build -r

println_blue "\nInstalling latest version of fastpasta from crates.io"

cargo install fastpasta --locked

println_blue "\nInstalling hyperfine for benchmarking"

cargo install hyperfine --locked

println_blue "\nInstalling binmult for effeciently copying file contents to sizes appropriate for benchmarking"

cargo install binmult --locked

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

## Make a temporary subdirectory for the test data (`binmult` will make+copy the larger files to this directory)
FILE_PATH="tests/test-data/"
tmp_file_path="${FILE_PATH}tmp/"
mkdir ${tmp_file_path}

# This is how much we'll ask `binmult` to "grow" the test files
BENCHMARK_FILE_SIZE_MIB=50
println_blue "Growing all test files to approximately ${BENCHMARK_FILE_SIZE_MIB} MiB with binmult\n\n"


# Files used in benchmarks
## Original files before they are `grown` to a reasonable size for benchmarking
PRE_TESTS_FILES_ARRAY=(
    "10_rdh.raw"
    "readout.superpage.1.raw"
    "tdh_no_data_ihw.raw"
    "rawtf_epn180_l6_1.raw"
)

tests_files_array=()

# Prepare more appropriate file sizes:
for file in "${PRE_TESTS_FILES_ARRAY[@]}"; do

    binmult "${FILE_PATH}${file}" --output "${tmp_file_path}${file}" --size "${BENCHMARK_FILE_SIZE_MIB}"

    tests_files_array+=("${tmp_file_path}${file}")

done


# Stores output of each test, from which the benchmark result is extracted and evaluated.
BENCH_RESULTS_FILE_PATH="bench_comp.md"
# Regex to extract the mean timings for each tested version of fastpasta (works on the markdown output of a `hyperfine` benchmark comparison)
REGEX_MEAN_TIMINGS="(?<=\` \| )\d*(?=\.)"
# Stores the mean timings of the local fastpasta vs. the remote (negative values -> the local is faster)
bench_results_local_mean_diff=()

test_cmds_array=(
    "check sanity"
    "check sanity its"
    "check all"
    "check all its"
    "check all its-stave"
)

local_pre="target/release/fastpasta"
remote_pre="fastpasta"

declare -a mean_timings
function bench_check_all_its_stave {
    local_cmd=$1; remote_cmd=$2;
    hyperfine \
        "${local_cmd} --mute-errors" \
        "${remote_cmd} --mute-errors" \
        --warmup 3\
        --style full\
        --time-unit millisecond\
        --shell=bash\
        --export-markdown ${BENCH_RESULTS_FILE_PATH}

    readarray -t timing_res < <( cat ${BENCH_RESULTS_FILE_PATH} | grep -Po "${REGEX_MEAN_TIMINGS}" | head -n 2 )
    mean_timings[0]=${timing_res[0]}
    mean_timings[1]=${timing_res[1]}
}


file_count=${#tests_files_array[@]}
cmd_count=${#test_cmds_array[@]}
total_test_count=$(( file_count * cmd_count ))
completed_tests=0


println_blue "Running ${cmd_count} command on each of ${file_count} files for a total of ${total_test_count} benchmarks"

for file in "${tests_files_array[@]}"; do

    for command in "${test_cmds_array[@]}"; do

        println_blue "\n---\nStarting test $(( completed_tests + 1 ))/${total_test_count}"

        println_magenta "\n ==> Benchmarking file ${file} with command: ${command}\n"

        bench_check_all_its_stave "${local_pre} ${file} ${command}" "${remote_pre} ${file} ${command}"
        local_mean=${mean_timings[0]}; remote_mean=${mean_timings[1]};
        local_minus_remote=$(( local_mean - remote_mean))
        bench_results_local_mean_diff+=("${local_minus_remote}")

        evaluate_benchmark_test_result "$local_mean" "$remote_mean"

        completed_tests=$(( completed_tests + 1 ))

    done

done


println_magenta "*********************************************************************************"
println_magenta "***                  SUMMARY OF PERFORMANCE REGRESSION TESTS                  ***"
println_magenta "*********************************************************************************\n"

total_diff=0
for i in "${bench_results_local_mean_diff[@]}"; do
    (( total_diff+=i ))
done

total_test_count=${#bench_results_local_mean_diff[@]}
println_magenta "Total timing difference in ${total_test_count} tests: ${total_diff} ms"

avg_diff=$(awk -v sum=$total_diff -v total_tests="${total_test_count}" 'BEGIN { print sum/total_tests }')
println_magenta "Average timing difference: ${avg_diff} ms"

println_cyan "\n--- RESULT --- \n"

if [[ $(float_cmp "${avg_diff}" 0) == 0 ]]; then
    println_blue "No difference"
elif [[ $(float_cmp "${avg_diff}" 0) -eq 2 ]]; then
    println_green "Nice! Seems faster overall!"
elif [[ $(float_cmp "${avg_diff}" 100) -eq 1 ]]; then
    println_red "This is really bad... D:"
else
    println_bright_yellow  "It seems slower but not significant"
fi

# Clean up
## Remove the benchmark output file.
rm ${BENCH_RESULTS_FILE_PATH}
## Remove temporary directory with the temporary `grown` raw data files.
rm -rf ${tmp_file_path}
