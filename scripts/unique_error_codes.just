# Parse all files in /src for error codes and check that there are no duplicates (content after #[cfg(test)] is excluded)
[private]
check-unique-error-codes:
    #!/usr/bin/env bash
    set -euo pipefail
    design_code=$( just get-design-code )
    # Pipe to grep: Match pattern and output exact match content (error codes)
    error_codes=$( echo ${design_code} | grep --perl-regexp --only-matching "E[0-9]{2,4}" )
    unique_error_codes=$( just filter-for-duplicates "${error_codes}" )
    # Pipe to grep: count the number of lines with `E` (there will just be an empty line if there's duplicate error codes)
    declare -i unique_error_codes_count=$( echo "${unique_error_codes}" | grep E --count )
    if [[ ${unique_error_codes_count} != 0 ]]; then
        {{PRINT}} red "Duplicate error codes found!"
        {{PRINT}} yellow "${unique_error_codes}"
        exit 1
    fi

# Get all fastpasta source/design code (not test code)
[private]
@get-design-code:
    # find: Find all files in `src/` and give them to `sed`.
    # sed: Read all file content until the string "cfg(test)" (delimits design and test code)
    find fastpasta/src/ -type f -exec sed -e "/cfg(test)/,\$d" {} \;

# Pipe to uniq: Output all duplicate lines
[private]
@filter-for-duplicates TEXT:
    echo "{{ TEXT }}" | uniq --repeated --count



