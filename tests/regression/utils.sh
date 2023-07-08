#!/bin/bash
# Contains utility functions for bash scripts

# Printing utility

TXT_CLEAR="\e[0m"

TXT_YELLOW="\e[33m"
function println_yellow {
    printf "${TXT_YELLOW}${1}${TXT_CLEAR}\n"
}

TXT_BRIGHT_CYAN="\e[96m"
function println_cyan {
    printf "${TXT_BRIGHT_CYAN}${1}${TXT_CLEAR}\n"
}

TXT_RED="\e[31m"
function println_red {
    printf "${TXT_RED}${1}${TXT_CLEAR}\n"
}

TXT_GREEN="\e[32m"
function println_green {
    printf "${TXT_GREEN}${1}${TXT_CLEAR}\n"
}

TXT_BRIGHT_GREEN="\e[92m"
function println_bright_green {
    printf "${TXT_BRIGHT_GREEN}${1}${TXT_CLEAR}\n"
}

TXT_BLUE="\e[34m"
function println_blue {
    printf "${TXT_BLUE}${1}${TXT_CLEAR}\n"
}

TXT_BRIGHT_MAGENTA="\e[95m"
function println_magenta {
    printf "${TXT_BRIGHT_MAGENTA}${1}${TXT_CLEAR}\n"
}

TXT_BRIGHT_YELLOW="\e[93m"
function println_bright_yellow {
    printf "${TXT_BRIGHT_YELLOW}${1}${TXT_CLEAR}\n"
}

# WARNING: Does not correctly compare between two negative floats. Not relevant for the use case though...
# Compares two floating points in bash without external dependencies such as `bc` or `awk`
# Returns `0` if they are equal, `1` if the first argument is greater, `2` if the second argument is.
## Very useful for a dystopian world where basic tools like `bc` are not available for an OS
## An example of such a world is Windows in 2023... (and Git Bash on windows)
function float_cmp() {
    # Floating-point numbers as arguments
    float_a="$1"; float_b="$2"

    # Extract integer and fractional parts
    int_a="${float_a%.*}"; frac_a="${float_a#*.}"
    int_b="${float_b%.*}"; frac_b="${float_b#*.}"

    # Check if only one argument has a negative sign
    if [[ ($float_a == -* && $float_b != -*) || ($float_a != -* && $float_b == -*) ]]; then
        if [[ $float_a == -* ]]; then
            echo "2" # b is greater
        else
            echo "1" # a is greater
        fi
        return
    fi

    # Compare integer parts
    if (( int_a > int_b )); then
        echo "1" # a is greater
    elif (( int_a < int_b )); then
        echo "2" # b is greater
    else
        # Compare fractional parts
        len_a="${#frac_a}"
        len_b="${#frac_b}"
        max_len=$(( len_a > len_b ? len_a : len_b ))
        frac_a_padded=$(printf "%-${max_len}s" "$frac_a")
        frac_b_padded=$(printf "%-${max_len}s" "$frac_b")

        if (( frac_a_padded > frac_b_padded )); then
            echo "1" # a is greater
        elif (( frac_a_padded < frac_b_padded )); then
            echo "2" # b is greater
        else
            echo "0" # a == b
        fi
    fi
}



function evaluate_benchmark_test_result {
    local_mean_timing=${1}; remote_mean_timing=${2}
    println_yellow "\n\tLocal fastpasta timing : ${local_mean_timing} ms (mean)"
    println_yellow "\n\tRemote fastpasta timing: ${remote_mean_timing} ms (mean)"

    local_minus_remote=$((${local_mean_timing}-${remote_mean_timing}))
    remote_minus_local=$((${remote_mean_timing}-${local_mean_timing}))

    bench_results_local_mean_diff+=(${local_minus_remote})
    println_bright_yellow "\t\tdifference between local and remote build: ${bench_results_local_mean_diff[@]} ms"

    if [[ "${local_mean_timing}" -lt ${remote_mean_timing} ]]; then
        println_green "\t\t-> local build is faster by ${remote_minus_local} ms!"
    elif [[ "${local_mean_timing}" -gt ${remote_mean_timing} ]]; then
        println_red "\t\t-> local build is slower by ${local_minus_remote} ms..."
    elif [[ "${local_mean_timing}" -eq ${remote_mean_timing} ]]; then
        println_blue "\t\t-> local and remote are about equally fast"
    fi
}
