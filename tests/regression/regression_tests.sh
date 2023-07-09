#!/bin/bash
###########
#### This script runs the binary fastPASTA and compares the output to the expected output
####
#### The output is compared using (case insensitive) regex patterns, counting the lines that match the pattern
####
#### CI Tip: Make shell scripts executable on CI with `git update-index --chmod=+x regression_tests.sh`
####
###########

# Utility functions
# shellcheck disable=SC1091
source ./tests/regression/utils.sh

# Prefix for each command.
## Run the binary and go to the test-data folder
CMD_PREFIX="cargo run -- ./tests/test-data/"

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
failed_output=()

### Regex patterns ###

## Matches the EOF and Exit Successful messages
## Needs -v2 to show "INFO -" prints
REGEX_EOF="(INFO -).*EOF"
REGEX_EXIT_SUCCESS="(DEBUG -).*Exit success"
## Matches the RDHs in the `view rdh` command, by going from the `:` in the memory offset to the version, header size, and data format.
REGEX_RDHS_IN_RDH_VIEW=": .* (7|6) .* 64 .* (0|2)"

# Tests are structured in an array of arrays
# Each test is an array of at least 3 elements
tests_array=(
    test_1_0 test_1_1 test_1_2 test_1_3 test_1_4 test_1_5 test_1_multi_0 test_1_multi_1
    test_2_0 test_2_multi_0 test_2_1 test_2_multi_1
    test_3_0 test_3_1 test_3_2 test_3_3 test_3_multi_0
    test_bad_ihw_tdh_detect_invalid_ids
    test_bad_dw_ddw0_detect_invalid_ids
    test_bad_tdt_detect_invalid_id
    test_bad_cdp_structure test_bad_cdp_structure_view_rdh test_bad_cdp_structure_detected
    test_bad_its_payload test_bad_its_payload_errors_detected
    test_thrs_cdw_3_links
)
# The 3 elements of a test is:
# 0: Command to run
# 1: Regex to match against stdout
# 2: Number of matches expected
# (optional) repeat pattern and number of matches for each additional test on the same file and command

### Tests on the `readout.superpage.1.raw` file
## Test 1_0: `check sanity` - Check that the program reached EOF and exits successfully
test_1_0=(
    "readout.superpage.1.raw check sanity -v3"
    "${REGEX_EOF}"
    1
    "${REGEX_EXIT_SUCCESS}"
    1
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
    "Trigger Type.*0x4813"
    1
    "Trigger Type.*HB"
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
    "$REGEX_RDHS_IN_RDH_VIEW"
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
    "10_rdh.raw check sanity -v3"
    "${REGEX_EOF}"
    1
    "${REGEX_EXIT_SUCCESS}"
    1
)
## Test 2_multi_0: `check sanity`
test_2_multi_0=(
    "10_rdh.raw check sanity"
    # Check the right run trigger type is detected
    "Trigger Type.*0x6A03"
    1
    # Check the right description of the trigger is printed
    "Trigger Type.*SOC"
    1
    # Check the right RDH version is detected
    "RDH.*Version.*7"
    1
    # Check the right number of RDHs is detected
    "Total.*RDHs.*10"
    1
    # Check the right number of HBFs is detected
    "Total.*hbfs.*5"
    1
    # Check the right layers and staves are detected
    "((layers)|(staves)).*((layers)|(staves)).*L0_12"
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
    "$REGEX_RDHS_IN_RDH_VIEW"
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
    "err_not_hbf.raw check sanity -v3"
    "${REGEX_EOF}"
    1
    "${REGEX_EXIT_SUCCESS}"
    1
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
    "$REGEX_RDHS_IN_RDH_VIEW"
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

### Tests on the 1_hbf_bad_cdp_structure.raw file
###
### This file contains a single HBF with a CDP with invalid structure because the RDH preceding a DDW0 does not have stop bit set
test_bad_cdp_structure=(
    "1_hbf_bad_cdp_structure.raw check sanity its -v3"
    # Check the file is parsed successfully
    "${REGEX_EOF}"
    1
    "${REGEX_EXIT_SUCCESS}"
    1
    # Check the error is not detected as this is just a sanity check.
    "error -"
    0
    "Total Errors.*0"
    1
    "Total RDHs.*2"
    1
    "Total HBFs.*0"
    1
)

test_bad_cdp_structure_view_rdh=(
    "1_hbf_bad_cdp_structure.raw view rdh"
    # Check the view contains 2 RDHs
    "$REGEX_RDHS_IN_RDH_VIEW"
    2
)

test_bad_cdp_structure_detected=(
    "1_hbf_bad_cdp_structure.raw check all its"
    # Check the error is detected
    "error - 0xE0: \[E..\].*RDH.*stop.bit"
    1
    "Total Errors.*1"
    1
)

### Tests on the 1_hbf_bad_its_payload.raw file
###
### This file contains a single HBF with an invalid ITS payload, containing 2 errors:
###     - an IHW ID comes instead of the TDH that should come after the first IHW.
###     - The IHW does not have lane 8 set in the active_lanes field, so data from lane 8 should generate an error
test_bad_its_payload=(
    "1_hbf_bad_its_payload.raw check sanity its -v3"
    # Check the file is parsed successfully
    "${REGEX_EOF}"
    1
    "${REGEX_EXIT_SUCCESS}"
    1
    # Check the error with 2 IHWs in a row is detected
    "error - 0x50: \[E..\].*ID"
    1
    # Check that it is the only error detected, by using negated character classes (should probably just start support lookarounds)
    "error - 0x([^5].|5[^0]):"
    0
)

test_bad_its_payload_errors_detected=(
    "1_hbf_bad_its_payload.raw check all its"
    # Check the invalid 2nd IHW is detected
    "error - 0x50: \[E..\].*ID"
    1
    # Check the error that there's data from lane 8 but the IHW does not have lane 8 set as active
    "error - 0x70: \[E..\].*lane 8.*IHW" # Checks that the error is detected the right place, and that it has something with lane 8 and IHW
    1
    "Total Errors.*2"
    1
)

### Tests on the thrs_cdw_links.raw file
###
### This file contains raw data from an IBS threshold scan, from link 8, 9, and 11.
test_thrs_cdw_3_links=(
    "thrs_cdw_links.raw check sanity its -v3"
    # Check the file is parsed successfully
    "${REGEX_EOF}"
    1
    "${REGEX_EXIT_SUCCESS}"
    1
    # Confirm no error count
    "Total errors.*0"
    1
    # Check that there are no errors
    "error -"
    0
    # Confirm RDH count
    "Total RDHs.*6"
    1
    # Confirm HBF count
    "Total HBFs.*3"
    1
    # Check that the 3 links are observed
    "Links observed.*8"
    1
    "Links observed.*9"
    1
    "Links observed.*11"
    1
)




# Run a single test
function run_test {
    test_var_name=$1
    test_case=$2
    pattern=$3
    cond=$4
    echo -e "==> running ${TXT_BRIGHT_MAGENTA}${test_var_name}${TXT_CLEAR}: ${TXT_BRIGHT_YELLOW}${test_case}${TXT_CLEAR}"
    echo -e "\tCondition is: ${TXT_BLUE}[number of matches] == ${cond}${TXT_CLEAR}, for pattern: ${TXT_BRIGHT_CYAN}${pattern}${TXT_CLEAR}"
    # Run the test, redirecting stderr to stdout, and skipping the first 2 lines (which are the "Finished dev..., Running..." lines)
    test_out=$(eval "${CMD_PREFIX}${test_case}" 2>&1 | tail -n +3 )
    # Count the number of matches
    matches=$(echo "${test_out}" | grep -E -i -c "${pattern}")
    # Check if the number of matches is the same as the expected number of matches
    if (( "${matches}" == "${cond}" ));
    then
        println_green "Test passed\n"
    else
        println_red "Test failed\n"
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
        run_test "$1" "$test_case" "$pattern" "$cond"
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
    how_many_tests_in_test "$test"
    total_tests=$((total_tests + $?))
done

println_magenta "************ REGRESSION TESTS ************"
println_magenta "***                                    ***"
printf "%b      Running %b regression tests   %b\n" "${TXT_BRIGHT_MAGENTA}***${TXT_CLEAR}" "${TXT_BRIGHT_YELLOW}${total_tests}${TXT_CLEAR}" "${TXT_BRIGHT_MAGENTA}***${TXT_CLEAR}"
println_magenta "***                                    ***"
println_magenta "******************************************\n\n"
# Run the tests
for test in "${tests_array[@]}"; do
    do_tests "$test"
done

println_magenta "************ END OF REGRESSION TESTS ************\n"

if  (( "${#failed_tests[@]}" == 0 ));
then
    println_bright_green "ALL ${total_tests} TESTS PASSED! :)"
    exit 0
else
    println_red "${#failed_tests[@]} Failed test(s):"
    for (( i = 0; i < ${#failed_tests[@]}; i++ )); do
        declare -n failed_test=${failed_tests[i]}
        println_red "${failed_tests[i]}${TXT_CLEAR}: ${failed_test[0]}"
        echo -e "${TXT_BRIGHT_CYAN}Pattern: ${TXT_CLEAR}${failed_patterns[i]}"
        echo -e "${TXT_BRIGHT_YELLOW}Expected:${TXT_CLEAR} ${failed_expected_matches[i]} ${TXT_BRIGHT_YELLOW}Got:${TXT_CLEAR} ${failed_matches[i]}"
        println_magenta "Test output:"
        echo -e "${failed_output[i]}"
    done
    exit 1
fi
