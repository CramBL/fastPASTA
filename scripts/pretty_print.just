#!/usr/bin/env -S just --justfile
## Cheatsheet: https://en.wikipedia.org/wiki/ANSI_escape_code#Colors

## Contains utilities for printing colored text to the terminal
##
## The "PRINT" variable is defined in the top-level justfile
##  and is an absolute path to this file:
## PRINT := join(justfile_directory(), "just-util/pretty_print.just")
##  thus it can be used to call the print function from any justfile in the project, in any directory,
##  making it immune to cd-ing around in Bash/Python/etc. recipes.
##
## Usage:
##    {{PRINT}} green "Success!"
##    {{PRINT}} cyan "Info"

ANSI_ESC_CLR := '\x1b[0m'

ANSI_BOLD_GREEN := '\x1b[1;32m'
ANSI_BOLD_CYAN := '\x1b[1;36m'
ANSI_BOLD_YELLOW := '\x1b[1;33m'
ANSI_BOLD_RED := '\x1b[1;31m'

# Green bold text with black background - good for success
[private]
green TEXT:
    #!/usr/bin/env bash
    printf "%b%b%b\n" "{{ANSI_BOLD_GREEN}}" "{{TEXT}}" "{{ANSI_ESC_CLR}}"

# Cyan bold text - good for info
[private]
cyan TEXT:
    #!/usr/bin/env bash
    printf "%b%b%b\n" "{{ANSI_BOLD_CYAN}}" "{{TEXT}}" "{{ANSI_ESC_CLR}}"

[private]
yellow TEXT:
    #!/usr/bin/env bash
    printf "%b%b%b\n" "{{ANSI_BOLD_YELLOW}}" "{{TEXT}}" "{{ANSI_ESC_CLR}}"

[private]
red TEXT:
    #!/usr/bin/env bash
    printf "%b%b%b\n" "{{ANSI_BOLD_RED}}" "{{TEXT}}" "{{ANSI_ESC_CLR}}"
