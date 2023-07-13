#!/bin/bash
###########
####
#### Parse all files in /src for error codes and check that there are no duplicates (content after #[cfg(test)] is excluded)
####
#### CI Tip: Make shell scripts executable on CI with `git update-index --chmod=+x script.sh`
###########

# Utility functions
# shellcheck disable=SC1091
source ./tests/regression/utils.sh

# find: Find all files in `src/` and give them to `sed`.
# sed: Read all file content until the string "cfg(test)" (delimits design and test code)
design_code=$( find src/ -type f -exec sed -e "/cfg(test)/,\$d" {} \; )
# Pipe to grep: Match pattern and output exact match content (error codes)
error_codes=$( echo "${design_code}" | grep -Po "E[0-9]{2,4}" )
# Pipe to uniq: Output all duplicate lines
unique_error_codes=$( echo "${error_codes}" | uniq -d -c )
# Pipe to grep: count the number of lines with `E` (there will just be an empty line if there's duplicate error codes)
unique_error_codes_count=$( echo "${unique_error_codes}" | grep E -c)

if [[ ${unique_error_codes_count} == 0 ]]; then
    println_bright_green "Error codes are unique as they should be"
    exit 0

else
    println_red "Duplicate error codes found!"
    println_bright_yellow "${unique_error_codes}"
    exit 1
fi
