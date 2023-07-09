#!/bin/bash
# Contains utility functions for bash scripts

# Printing utility

## (reverts to default color)
readonly TXT_CLEAR="\e[0m"; export TXT_CLEAR

readonly TXT_YELLOW="\e[33m"; export TXT_YELLOW
function println_yellow {
    printf  "\e[33m%b\e[0m\n" "${1}"
}

readonly TXT_BRIGHT_CYAN="\e[96m"; export TXT_BRIGHT_CYAN
function println_cyan {
    printf "\e[96m%b\e[0m\n" "${1}"
}

readonly TXT_RED="\e[31m"; export TXT_RED
function println_red {
    printf "\e[31m%b\e[0m\n" "${1}"
}

readonly TXT_GREEN="\e[32m"; export TXT_GREEN
function println_green {
    printf "\e[32m%b\e[0m\n" "${1}"
}

readonly TXT_BRIGHT_GREEN="\e[92m"; export TXT_BRIGHT_GREEN
function println_bright_green {
    printf "\e[92m%b\e[0m\n" "${1}"
}

readonly TXT_BLUE="\e[34m"; export TXT_BLUE
function println_blue {
    printf "\e[34m%b\e[0m\n" "${1}"
}

readonly TXT_BRIGHT_MAGENTA="\e[95m"; export TXT_BRIGHT_MAGENTA
function println_magenta {
    printf "\e[95m%b\e[0m\n" "${1}"
}

readonly TXT_BRIGHT_YELLOW="\e[93m"; export TXT_BRIGHT_YELLOW
function println_bright_yellow {
    printf "\e[93m%b\e[0m\n" "${1}"
}

# WARNING: Does not correctly compare between two negative floats. Not relevant for the use case though...
# WARNING: Also incorrect if the integer parts are equal but the length of the fractional part of A and B are different.
# Compares two floating points in bash without external dependencies such as `bc` or `awk`
# Returns `0` if they are equal, `1` if the first argument is greater, `2` if the second argument is.
## Very useful for a dystopian world where basic tools like `bc` are not available for an OS
## An example of such a world is Windows in 2023... (and Git Bash on windows)
function float_cmp() {
    # Floating-point numbers as arguments
    local float_a="$1"
    local float_b="$2"

    # Extract integer and fractional parts
    local int_a="${float_a%.*}"; local frac_a="${float_a#*.}"
    local int_b="${float_b%.*}"; local frac_b="${float_b#*.}"

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
        local len_a="${#frac_a}"
        local len_b="${#frac_b}"
        local max_len=$(( len_a > len_b ? len_a : len_b ))
        local frac_a_padded
        frac_a_padded=$(printf "%-${max_len}s" "$frac_a")
        local frac_b_padded
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
    local -r local_mean_timing=${1}
    local -r remote_mean_timing=${2}

    println_yellow "\n\tLocal fastpasta timing : ${local_mean_timing} ms (mean)"
    println_yellow "\n\tRemote fastpasta timing: ${remote_mean_timing} ms (mean)"

    local -r local_minus_remote=$(( local_mean_timing - remote_mean_timing ))
    local -r remote_minus_local=$(( remote_mean_timing - local_mean_timing ))

    println_bright_yellow "\t\tdifference between local and remote build: ${local_minus_remote} ms"

    local diff_percent
    diff_percent=$( calc_relative_percent "${local_mean_timing}" "${remote_mean_timing}" )

    if [[ "${local_mean_timing}" -lt ${remote_mean_timing} ]]; then
        println_green "\t\t\t-> local build is faster by ${remote_minus_local} ms!"
        println_green "\t\t\t-> -${diff_percent: 1:4} %"

    elif [[ "${local_mean_timing}" -gt ${remote_mean_timing} ]]; then
        println_red "\t\t\t-> local build is slower by ${local_minus_remote} ms..."
        println_red "\t\t\t-> +${diff_percent: 0:4} %"

    elif [[ "${local_mean_timing}" -eq ${remote_mean_timing} ]]; then
        println_blue "\t\t\t-> local and remote are about equally fast"
    fi
}

function calc_average {
    local -r sum=$1
    local -r N=$2
    avg=$(awk -v sum="${sum}" -v total_tests="${N}" 'BEGIN { print sum/total_tests }')
    echo "$avg"
}

function calc_relative_percent {
    local -r absolute_time_a=$1
    local -r absolute_time_b=$2
    local -r delta=$(( absolute_time_a - absolute_time_b ))
    local rel_delta
    rel_delta=$( \
        awk\
            -v delta="${delta}"\
            -v relvar="${absolute_time_b}"\
                'BEGIN { \
                    x=delta/relvar
                    x=x*100
                    print x
                }'\
    )
    echo "$rel_delta"
}

function calc_relative_fraction {
    local -r absolute_time_a=$1
    local -r absolute_time_b=$2
    local rel_delta
    rel_delta=$( \
        awk\
            -v a="${absolute_time_a}"\
            -v b="${absolute_time_b}"\
                'BEGIN { \
                    x=a/b
                    print x
                }'\
    )
    echo "$rel_delta"
}

function fraction_to_percent {
    local -r fraction=$1
    local percent
    percent=$( \
        awk\
            -v x="${fraction}"\
                'BEGIN { \
                    print x*100
                }'\
    )
    echo "$percent"
}
