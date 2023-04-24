#!/bin/bash
###########
#### This script runs the binary fastPASTA and compares the output to the expected output
####
#### The output is compared using (case insensitive) regex patterns, counting the lines that match the pattern
####
#### CI Tip: Make shell scripts executable on CI with `git update-index --chmod=+x regression_tests.sh`
####
###########
TXT_RED="\e[31m"
TXT_YELLOW="\e[33m"
TXT_GREEN="\e[32m"
TXT_BLUE="\e[34m"
TXT_BRIGHT_YELLOW="\e[93m"
TXT_BRIGHT_CYAN="\e[96m"
TXT_BRIGHT_MAGENTA="\e[95m"
TXT_BRIGHT_GREEN="\e[92m"
TXT_CLEAR="\e[0m"

# Prefix for each command.
## Run the binary and go to the test-data folder
cmd_prefix="cargo run -- ./test-regression/test-data/"

# Arrays to store the failed tests
## The index of each array corresponds to the index of the failed test in the tests_array
failed_tests=()
## The patterns of each failed test
failed_patterns=()
## Expected number of matches of each failed test
failed_expected_matches=()
## The matches of each failed test
failed_matches=()
## The output of each failed test
failed_results=()

### Regex patterns ###

## Matches the EOF and Exit Successful messages
## Needs -v2 to show "INFO -" prints
re_eof="((INFO -).*((EOF)|(Exit Successful))*)"
## Matches the RDHs in the `view rdh` command, by going from the `:` in the memory offset to the version, header size, and data format.
re_rdhs_in_rdh_view=": .* (7|6) .* 64 .* (0|2)"

# Tests are structured in an array of arrays
# Each test is an array of at least 3 elements
tests_array=(
    test_1_0 test_1_1 test_1_2 test_1_3 test_1_4 test_1_5 test_1_multi_0 test_1_multi_1
    test_2_0 test_2_multi_0 test_2_1 test_2_multi_1
    test_3_0 test_3_1 test_3_2 test_3_3 test_3_multi_0
    test_bad_ihw_tdh_detect_invalid_ids
    test_bad_dw_ddw0_detect_invalid_ids
    test_bad_tdt_detect_invalid_id
)
# The 3 elements of a test is:
# 0: Command to run
# 1: Regex to match against stdout
# 2: Number of matches expected
# (optional) repeat pattern and number of matches for each additional test on the same file and command

### Tests on the `readout.superpage.1.raw` file
## Test 1_0: `check sanity` - Check that the program reached EOF and exits successfully
test_1_0=(
    "readout.superpage.1.raw check sanity -v2"
    "${re_eof}"
    2
)
## Test 1_1: `check sanity` - Check the right data format is detected
test_1_1=(
    "readout.superpage.1.raw check sanity"
    "Data Format.* 2"
    1
)
## Test 1_2: `check sanity its` - Check the right rdh version is detected
test_1_2=(
    "readout.superpage.1.raw check sanity its"
    "rdh version.*7"
    1
)
## Test 1_multi_0: `check all`
test_1_multi_0=(
    "readout.superpage.1.raw check all"
    # Check the right link is detected
    "links .*2"
    1
    # Check the right amount of RDHs is detected
    "total rdhs.*6"
    1
    # Check that no errors are detected
    "error - "
    0
)
## Test 1_5: `check all its` - Check the right amount of HBFs is detected
test_1_3=(
    "readout.superpage.1.raw check all its"
    "total hbfs.*3"
    1
)
## Test 1_6: `check sanity` - Check the right layers and staves are detected
test_1_4=(
    "readout.superpage.1.raw check sanity"
    "((layers)|(staves)).*((layers)|(staves)).*L1_6"
    1
)
## Test 1_7: `view rdh` - Check the right amount of RDHs is shown
test_1_5=(
    "readout.superpage.1.raw view rdh"
    "$re_rdhs_in_rdh_view"
    6
)
## Test 1_multi_1 `view hbf`
test_1_multi_1=(
    "readout.superpage.1.raw view hbf"
    # Check the right amount of IHWs is shown
    ": IHW "
    3
    # Check the right amount of TDH is shown
    ": TDH "
    3
    # Check the right amount of TDT is shown
    ": TDT "
    3
    # Check the right amount of DDW is shown
    ": DDW "
    3
)

# Tests on the `10_rdh.raw` file
## Test 2_0: sanity check that the program reached EOF and exits successfully
test_2_0=(
    "10_rdh.raw check sanity -v2"
    "${re_eof}"
    2
)
## Test 2_multi_0: `check sanity`
test_2_multi_0=(
    "10_rdh.raw check sanity"
    # Check the right RDH version is detected
    "RDH.*Version.*7"
    1
    # Check the right number of RDHs is detected
    "Total.*RDHs.*10"
    1
    # Check the right number of HBFs is detected
    "Total.*hbfs.*5"
    1
    # Check that no errors are detected
    "error - "
    0
    # Check that no warnings are generated
    "warn - "
    0
)
## Test 2_4: `view rdh` - Check the right number of RDHs is shown
test_2_1=(
    "10_rdh.raw view rdh"
    "$re_rdhs_in_rdh_view"
    10
)
## Test 2_multi_1: `view hbf` -
test_2_multi_1=(
    "10_rdh.raw view hbf"
    # Check the right number of RDHs is shown
    ": RDH"
    10
    # Check the right number of IHWs is shown
    ": IHW"
    5
    # Check the right number of TDHs is shown
    ": TDH"
    5
    # Check the right number of TDTs is shown
    ": TDT"
    5
    # Check the right number of DDWs is shown
    ": DDW"
    5
)


# Tests on the `err_not_hbf.raw` file
## Test 3_0: sanity check that the file is parsed successfully
test_3_0=(
    "err_not_hbf.raw check sanity -v2"
    "${re_eof}"
    2
)
## Test 3_1: `check all` - Check the right number of errors are detected
test_3_1=(
    "err_not_hbf.raw check all"
    "(error - 0xa0.*pages)|(Total Errors.*[0-9])"
    2
)
## Test 3_2: `check sanity` - Check the right number of errors are detected
test_3_2=(
    "err_not_hbf.raw check sanity"
    "error - "
    0
)
## Test 3_3: `view rdh` - Check the right number of RDHs is shown
test_3_3=(
    "err_not_hbf.raw view rdh"
    "$re_rdhs_in_rdh_view"
    2
)
## Test 3_multi_0: `view hbf`
test_3_multi_0=(
    "err_not_hbf.raw view hbf"
    # Check the right number of RDHs is shown
    ": RDH "
    2
    # Check the right number of IHWs is shown
    ": IHW "
    2
    # Check the right number of TDHs is shown
    ": TDH "
    2
    # Check the right number of TDTs is shown
    ": TDT "
    2
    # Check the right number of DDWs is shown
    ": DDW "
    0 # There are no DDWs in this file as it is an erroneous file
)

### Tests on the 1_hbf_bad_ihw_tdh.raw file
###
### This file contains a single HBF with an IHW with an invalid ID (0xE1) and a TDH with invalid ID (0xE9)
test_bad_ihw_tdh_detect_invalid_ids=(
    "1_hbf_bad_ihw_tdh.raw check sanity its"
    # Check the error is detected in the right position with the right error code and message
    "error - 0x40: \[E30\].*ID.*0xe1"
    1
    # Check the error is detected in the right position with the right error code and message
    "error - 0x50: \[E40\].*ID.*0xe9"
    1
)

### Tests on the 1_hbf_bad_dw_ddw0.raw file
###
### This file contains a single HBF with a Data word with invalid ID (0x1) a DDW0 with invalid ID (0xE5)
test_bad_dw_ddw0_detect_invalid_ids=(
    "1_hbf_bad_dw_ddw0.raw check sanity its"
    # Check the error is detected in the right position with the right error code and message
    # Should give an unregonized ID errors as it is in an ambigiuous position where several words could be valid
    # Checks that it ends with `01]` as it should print the GBT bytes which would end with the wrong ID (01)
    "error - 0x80: \[E99\] Unrecognized ID.*01\]"
    1
    # Same as above but for the DDW0 error
    "error - 0xE0: \[E99\] Unrecognized ID.*E5\]"
    1
    # Check that 2 Invalid ID errors are also detected in those two positions
    "error - (0x80|0xE0): \[E.0\].*ID" # Just checks its related to a sanity check regarding ID by checking the error code is Ex0
    2
)

### Tests on the 1_hbf_bad_tdt.raw file
###
### This file contains a single HBF with a TDT with invalid ID (0xF1)
test_bad_tdt_detect_invalid_id=(
    "1_hbf_bad_tdt.raw check sanity its"
    # Check the error is detected in the right position with the right error code and message
    "error - 0x90: \[E99\].*ID.*f1"
    1
)



# Run a single test
function run_test {
    test_var_name=$1
    test_case=$2
    pattern=$3
    cond=$4
    echo -e "running ${TXT_BRIGHT_MAGENTA}${test_var_name}${TXT_CLEAR}: ${TXT_BRIGHT_YELLOW}${test_case}${TXT_CLEAR}"
    echo -e "Condition is: ${TXT_BLUE}[number of matches] == ${cond}${TXT_CLEAR}, for pattern: ${TXT_BRIGHT_CYAN}${pattern}${TXT_CLEAR}"
    # Run the test, redirecting stderr to stdout, and skipping the first 2 lines (which are the "Finished dev..., Running..." lines)
    test_out=$(eval ${cmd_prefix}${test_case} 2>&1 | tail -n +3 )
    # Count the number of matches
    matches=$(echo "${test_out}" | egrep -i -c "${pattern}")
    # Check if the number of matches is the same as the expected number of matches
    if (( "${matches}" == "${cond}" ));
    then
        echo -e "${TXT_GREEN}Test passed${TXT_CLEAR}"
    else
        echo -e "${TXT_RED}Test failed${TXT_CLEAR}"
        # Add the test info to the failed tests arrays
        failed_tests+=("${test}")
        failed_patterns+=("${pattern}")
        failed_expected_matches+=("${cond}")
        failed_matches+=("${matches}")
        failed_output+=("${test_out}")
    fi
}

# Run all the tests in a test array
function do_tests {
    # The test array is passed by name, so we need to use declare -n to get the array
    declare -n test_arr=$1
    # The first element of the array is the test case (cmd)
    test_case=${test_arr[0]}
    # Run all the tests in the array (skipping the first element, which is the test case)
    for ((i=1; i<${#test_arr[@]}; i+=2)); do
        pattern=${test_arr[$i]}
        cond=${test_arr[$((i+1))]}
        run_test $1 "$test_case" "$pattern" "$cond"
    done
}

# Calculate the total number of tests in a test array
function how_many_tests_in_test {
    declare -n test_arr=$1
    # The number of tests is half the number of elements in the array minus 1 (the test case)
    return $(( (${#test_arr[@]} - 1) / 2 ))
}

### Main ###

total_tests=0
for test in "${tests_array[@]}"; do
    how_many_tests_in_test $test
    total_tests=$((total_tests + $?))
done
echo -e "Running ${TXT_BRIGHT_YELLOW}${total_tests}${TXT_CLEAR} regression tests"

# Run the tests
for test in "${tests_array[@]}"; do
    do_tests $test
done

echo
if  (( "${#failed_tests[@]}" == 0 ));
then
    echo -e "${TXT_BRIGHT_GREEN}ALL TESTS PASSED! :)${TXT_CLEAR}"
    exit 0
else
    echo -e "${TXT_RED}${#failed_tests[@]} Failed test(s):${TXT_CLEAR}"
    for (( i = 0; i < ${#failed_tests[@]}; i++ )); do
        declare -n failed_test=${failed_tests[i]}
        echo -e "${TXT_RED}${failed_tests[i]}${TXT_CLEAR}: ${failed_test[0]}"
        echo -e "${TXT_BRIGHT_CYAN}Pattern: ${TXT_CLEAR}${failed_patterns[i]}"
        echo -e "${TXT_BRIGHT_YELLOW}Expected:${TXT_CLEAR} ${failed_expected_matches[i]} ${TXT_BRIGHT_YELLOW}Got:${TXT_CLEAR} ${failed_matches[i]}"
        echo -e "${TXT_BRIGHT_MAGENTA}Test output: ${TXT_CLEAR}"
        echo -e "${failed_output[i]}"
    done
    exit 1
fi
