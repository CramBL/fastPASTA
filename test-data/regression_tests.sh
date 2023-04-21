#!/bin/bash

# This script runs the binary fastPASTA and compares the output to the expected output

TXT_RED="\e[31m"
TXT_YELLOW="\e[33m"
TXT_GREEN="\e[32m"
TXT_BLUE="\e[34m"
TXT_BRIGHT_YELLOW="\e[93m"
TXT_BRIGHT_CYAN="\e[96m"
TXT_BRIGHT_MAGENTA="\e[95m"
TXT_BRIGHT_GREEN="\e[92m"
TXT_CLEAR="\e[0m"
failed_tests=()
failed_matches=()
failed_results=()

# Tests are structured in an array of arrays
# Each test is an array of 3 elements
tests_array=(test1 test2 test3 test4 test5 test6 test7 test8 test9 test10 test11 test12 test13 test14)
# The 3 elements of a test is:
# 0: Command to run
# 1: Regex to match against stdout
# 2: Number of matches expected

# Tests on the `readout.superpage.1.raw` file
## Test 1: `check all its` - Check the right data format is detected, and that the program reached EOF and exits successfully
test1=(
    "cargo run -- ./test-data/readout.superpage.1.raw check all its -v2"
    "Data Format.*2|((INFO -).*((EOF)|(Exit Successful))*)"
    3
)
## Test 2: `check sanity its` - Check the right data format is detected
test2=(
    "cargo run -- ./test-data/readout.superpage.1.raw check sanity its"
    "Data Format.*2"
    1
)

# Tests on the `10_rdh.raw` file
## Test 3: `check sanity` - Check the right RDH version is detected
test3=(
    "cargo run -- ./test-data/10_rdh.raw check sanity"
    "RDH.*Version.*7"
    1
)
## Test 4: `check sanity` - Check the right number of RDHs is detected
test4=(
    "cargo run -- ./test-data/10_rdh.raw check sanity"
    "Total.*RDHs.*10"
    1
)
## Test 5: `check sanity` - Check the right number of HBFs is detected
test5=(
    "cargo run -- ./test-data/10_rdh.raw check sanity"
    "Total.*hbfs.*5"
    1
)
## Test 6: `view hbf` - Check the right number of RDHs is shown
test6=(
    "cargo run -- ./test-data/10_rdh.raw view hbf"
    "RDH"
    10
)
## Test 7: `view rdh` - Check the right number of RDHs is shown
test7=(
    "cargo run -- ./test-data/10_rdh.raw view rdh"
    "7.*64.*8"
    10
)

# Tests on the `err_not_hbf.raw` file
## Test 8: `check all` - Check the right number of errors are detected
test8=(
    "cargo run -- ./test-data/err_not_hbf.raw check all"
    "(error - 0xa0.*pages)|(Total Errors.*[0-9])"
    2
)
## Test 9: `check sanity` - Check the right number of errors are detected
test9=(
    "cargo run -- ./test-data/err_not_hbf.raw check sanity"
    "error - "
    0
)
## Test 10: `view rdh` - Check the right number of RDHs is shown
test10=(
    "cargo run -- ./test-data/err_not_hbf.raw view rdh"
    "7.*64.*8"
    2
)
## Test 11: `view hbf` - Check the right number of RDHs is shown
test11=(
    "cargo run -- ./test-data/err_not_hbf.raw view hbf"
    "RDH"
    2
)
## Test 12: `view hbf` - Check the right number of IHWs is shown
test12=(
    "cargo run -- ./test-data/err_not_hbf.raw view hbf"
    "IHW"
    2
)
## Test 13: `view hbf` - Check the right number of TDHs is shown
test13=(
    "cargo run -- ./test-data/err_not_hbf.raw view hbf"
    "TDH"
    2
)
## Test 14: `view hbf` - Check the right number of TDTs is shown
test14=(
    "cargo run -- ./test-data/err_not_hbf.raw view hbf"
    "TDT"
    2
)

echo -e "Running ${TXT_BRIGHT_YELLOW}${#tests_array[@]}${TXT_CLEAR} regression tests"

for test in "${tests_array[@]}"; do
    declare -n current_test=$test
    test_case=${current_test[0]}
    pattern=${current_test[1]}
    cond=${current_test[2]}
    echo -e "running ${TXT_BRIGHT_MAGENTA}${test}${TXT_CLEAR}: ${TXT_BRIGHT_YELLOW}${test_case}${TXT_CLEAR}"
    echo -e "Condition is: ${TXT_BLUE}[number of matches] == ${cond}${TXT_CLEAR}, for pattern: ${TXT_BRIGHT_CYAN}${pattern}${TXT_CLEAR}"
    test_out=$(eval ${test_case} 2>&1)
    # `tail -n +3` starts from line 3, thus skipping the 2 lines saying "Finished dev..., Running..."
    matches=$(echo "${test_out}" | tail -n +3 | egrep -i -c "${pattern}")
    #echo -e "matches:${matches}";
    if (( "${matches}" == "${cond}" ));
    then
        echo -e "${TXT_GREEN}Test passed${TXT_CLEAR}"
    else
        echo -e "${TXT_RED}Test failed${TXT_CLEAR}"
        failed_tests+=("${test}")
        failed_matches+=("${matches}")
        failed_output+=("${test_out}")
    fi;
done

echo
if  [[ "${#failed_tests[@]}" == 0 ]];
then
    echo -e "${TXT_BRIGHT_GREEN}ALL TESTS PASSED! :)${TXT_CLEAR}"
    exit 0
else
    echo -e "${TXT_RED}${#failed_tests[@]} Failed test(s):${TXT_CLEAR}"
    for (( i = 0; i < ${#failed_tests[@]}; i++ )); do
        declare -n failed_test=${failed_tests[i]}
        echo -e "${TXT_RED}${failed_tests[i]}${TXT_CLEAR}: ${failed_test[0]}"
        echo -e "${TXT_BRIGHT_CYAN}Pattern: ${TXT_CLEAR}${failed_test[1]}"
        echo -e "${TXT_BRIGHT_YELLOW}Expected:${TXT_CLEAR} ${failed_test[2]} ${TXT_BRIGHT_YELLOW}Got:${TXT_CLEAR} ${failed_matches[i]}"
        echo -e "${TXT_BRIGHT_MAGENTA}Test output: ${TXT_CLEAR}"
        echo -e "${failed_output[i]}"
    done
    exit 1
fi
