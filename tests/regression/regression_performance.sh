#!/bin/bash
###########
#### This script runs benchmarks of the local binary vs. the latest released one that is installed with `cargo install fastpasta`
####
#### The files used for benchmarking is the same that are used during other system/regression tests.
####    To get a file size appropriate for benchmarks, the `binmult` binary is installed and used to `grow` the existing
####    files by copying them and duplicating them by appending them to themselves until they reach the desired size
###     these files are deleted at the end of the benchmarks.
####
#### CI Tip: Make shell scripts executable on CI with `git update-index --chmod=+x regression_tests.sh`
####
###########

# Import utility functions
# shellcheck disable=SC1091
source ./tests/regression/utils.sh

set -euo pipefail
#set -x # Uncomment if debugging
# Echo each line if running in CI
if [ -n "${CI-}" ]; then
    set -x
fi

# Minimum runs hyperfine performs to benchmark a given command
declare -i MIN_RUNS=20

# This is how much we'll ask `binmult` to "grow" the test files to in MiB
declare -i BENCHMARK_FILE_SIZE_MIB=300

# Files used in benchmarks
## Original files before they are `grown` to a reasonable size for benchmarking
declare -a PRE_TESTS_FILES_ARRAY=(
    "10_rdh.raw"
    "12_links_2hbf.raw"
    "thrs_cdw_links.raw"
)


if [[ "${EXTENDED:-false}" != "false" ]]; then
    println_bright_yellow "Running benchmarks in EXTENDED mode\n"
    (( BENCHMARK_FILE_SIZE_MIB*=2 ))
    (( MIN_RUNS*=2 ))
fi

##### Constants #####
## Constant variables (not arrays)
readonly FILE_PATH="tests/test-data/"
readonly tmp_file_path="${FILE_PATH}tmp/"
# Stores output of each test, from which the benchmark result is extracted and evaluated.
readonly BENCH_RESULTS_FILE_PATH="bench_comp.md"
# Regex to extract the mean timings for each tested version of fastpasta (works on the markdown output of a `hyperfine` benchmark comparison)
readonly REGEX_MEAN_TIMINGS="(?<=\` \| )\d*(?=\.)"
# Prefixes for the running the local binary vs. released
readonly LOCAL_PRE="target/release/fastpasta"
readonly RELEASED_PRE="fastpasta"

## Constant arrays
test_cmd_args="${cmds:-check sanity; check all; check all its; check all its-stave}"

IFS=';' read -ra test_cmds_array <<< "${test_cmd_args}"

declare -i -r cmd_count=${#test_cmds_array[@]}

println_cyan "Benchmarking with ${cmd_count} command(s):"
for cmd in "${test_cmds_array[@]}"; do
    println_bright_yellow "\t- $(trim "${cmd}")"
done

##### Readonly variables generated from constants above #####

println_blue "\n == Toolchain versions =="

cargo --version
rustc --version

println_yellow "\nBuilding in release mode\n"

cargo build --release

println_blue "\nInstalling latest version of fastpasta from crates.io"

cargo install fastpasta --locked

println_blue "\nInstalling hyperfine for benchmarking"

cargo install hyperfine --locked

println_blue "\nInstalling binmult for effeciently copying file contents to sizes appropriate for benchmarking"

cargo install binmult --locked

println_cyan "\nChecking version of local fastpasta build"

target/release/fastpasta --version

println_cyan "Checking version of released fastpasta installation"

fastpasta --version

println_magenta "\n===================================================================================================== "
println_magenta "***********************************************************************************"
println_magenta "***                                                                             ***"
println_magenta "*** Benchmarking the local compiled binary vs. the latest released installation ***"
println_magenta "***                                                                             ***"
println_magenta "***********************************************************************************\n"

## Make a temporary subdirectory for the test data (`binmult` will make+copy the larger files to this directory)
mkdir -p ${tmp_file_path}

println_blue "Growing all test files to approximately ${BENCHMARK_FILE_SIZE_MIB} MiB with binmult\n\n"

declare -a tests_files_array=()
# Prepare more appropriate file sizes:
for file in "${PRE_TESTS_FILES_ARRAY[@]}"; do

    binmult "${FILE_PATH}${file}" --output "${tmp_file_path}${file}" --size "${BENCHMARK_FILE_SIZE_MIB}"

    tests_files_array+=("${tmp_file_path}${file}")

done

declare -i -r file_count=${#tests_files_array[@]}
declare -i -r total_test_count=$(( file_count * cmd_count ))

# Stores the mean absolute timings of the local fastpasta vs. the released (negative values -> the local is faster)
declare -a bench_results_local_mean_diff_absolute=()
# The accumulated execution time of the released version of fastpasta
declare -i bench_results_released_total_ms=0

# Make two arrays with as many 0's as there's test command
# Use this to store the diff of each test command and the absolute timing of the released version
declare -a test_cmds_diff=()
declare -a test_cmds_released_abs=()
for ((i=0; i<cmd_count; i++)); do
    test_cmds_diff+=(0)
    test_cmds_released_abs+=(0)
done


declare -a mean_timings
function bench_two_cmds_return_timings {
    local local_cmd=$1;
    local released_cmd=$2;

    hyperfine \
        "${local_cmd} --mute-errors --verbosity 0" \
        "${released_cmd} --mute-errors --verbosity 0" \
        --warmup 3\
        --style full\
        --time-unit millisecond\
        --shell=bash\
        --min-runs "${MIN_RUNS}"\
        --export-markdown ${BENCH_RESULTS_FILE_PATH}

    readarray -t timing_res < <( cat ${BENCH_RESULTS_FILE_PATH} | grep -Po "${REGEX_MEAN_TIMINGS}" | head -n 2 )

    mean_timings[0]=${timing_res[0]}
    mean_timings[1]=${timing_res[1]}
}

declare -i completed_tests=0

println_blue "Running ${cmd_count} command on each of ${file_count} files for a total of ${total_test_count} benchmarks"

for file in "${tests_files_array[@]}"; do

    declare -i test_cmd_idx=0
    for command in "${test_cmds_array[@]}"; do

        println_blue "\n---\nStarting test $(( completed_tests + 1 ))/${total_test_count}"

        println_magenta "\n ==> Benchmarking file ${file} with command: ${command}\n"

        bench_two_cmds_return_timings "${LOCAL_PRE} ${file} ${command}" "${RELEASED_PRE} ${file} ${command}"
        declare -i local_mean=${mean_timings[0]}
        declare -i released_mean=${mean_timings[1]}
        declare -i local_minus_released=$(( local_mean - released_mean))
        bench_results_local_mean_diff_absolute+=("${local_minus_released}")
        bench_results_released_total_ms=$(( bench_results_released_total_ms + released_mean ))
        # Store the absolute timing values for each command run by the released version
        (( test_cmds_released_abs[test_cmd_idx]+=released_mean ))
        (( test_cmd_idx+=1 ))

        evaluate_benchmark_test_result "$local_mean" "$released_mean"

        completed_tests=$(( completed_tests + 1 ))

    done
done

# Clean up the temporary files
## Remove the benchmark output file.
rm ${BENCH_RESULTS_FILE_PATH}
## Remove temporary directory with the temporary `grown` raw data files.
rm -rf ${tmp_file_path}

println_magenta "*********************************************************************************"
println_magenta "***                  SUMMARY OF PERFORMANCE REGRESSION TESTS                  ***"
println_magenta "*********************************************************************************\n"

### Calculate the total performance diff as well as the diff for each command
declare -i test_cmds_counter=0 # Loop counter to store diffs in the test command diff array
declare -i total_diff=0
for i in "${bench_results_local_mean_diff_absolute[@]}"; do
    (( total_diff+=i ))
    (( test_cmds_diff[test_cmds_counter]+=i ))
    (( test_cmds_counter+=1 ))
    if [[ ${test_cmds_counter} == ${cmd_count} ]]; then
        test_cmds_counter=0
    fi

done

println_blue "The released version of fastpasta took a total of ${bench_results_released_total_ms} ms [sum of means] across ${total_test_count} benchmarks\n"

println_magenta "Timing difference in ${total_test_count} tests:"
println_magenta "\tTotal: ${total_diff} ms"

avg_diff=$( calc_average $total_diff $total_test_count )
readonly avg_diff
frac_diff=$( calc_relative_fraction $total_diff $bench_results_released_total_ms)
readonly frac_diff
percent_diff=$( fraction_to_percent "$frac_diff" )
readonly percent_diff

println_magenta "\tMean : ${avg_diff} ms\n"

# Calculate and show the timing difference for each tested command
declare -i idx=0
for timing_diff in "${test_cmds_diff[@]}"; do

    #cmd_avg_diff=$( calc_average "${timing_diff}" $total_test_count )
    cmd_frac_diff=$( calc_relative_fraction "${timing_diff}" "${test_cmds_released_abs[idx]}")
    cmd_percent_diff=$( fraction_to_percent "${cmd_frac_diff}" )
    padded_timing_diff_str=$(left_pad_str "${timing_diff}" 10 ' ')
    padded_cmd_str=$(right_pad_str "$(trim "${test_cmds_array[idx]}")" 19 ' ')
    # Old fashioned '\t' because of the traling '$'
    println_bright_yellow "        ${padded_cmd_str}${padded_timing_diff_str} ms (${cmd_percent_diff: 0:5} %)"
    (( idx+=1 ))
    if [[ ${idx} == "${cmd_count}" ]]; then
        idx=0
    fi
done

# Unset echoing each line to make this printing much more clear
set +x
println_cyan "\n--- CONCLUSION --- \n"

if [[ $(float_cmp "${avg_diff}" 0) == 0 ]]; then
    println_blue "No difference"
    exit 0

elif [[ $(float_cmp "${avg_diff}" 0) -eq 2 ]]; then
    printf "Execution time: "
    println_green "-${percent_diff: 1:4} %"
    println_green "Nice! Seems faster overall!"
    exit 0

elif [[ $(float_cmp "${percent_diff}" 10) -eq 1 ]]; then
    printf "Execution time: "
    println_red "+${percent_diff: 0:4} %"
    println_red "SEVERE PERFORMANCE REGRESSION"
    println_red "High likelihood of frequent unnecessary allocation or even a bug!"
    exit 1

elif [[ $(float_cmp "${percent_diff}" 5) -eq 1 ]]; then
    printf "Execution time: "
    println_red "+${percent_diff: 0:4} %"
    println_red "This is really bad... Consider refactoring! :("
    exit 0

else
    printf "Execution time: "
    println_bright_yellow "+${percent_diff: 0:4} %"
    println_bright_yellow "It seems slower but not significantly"
    exit 0
fi
