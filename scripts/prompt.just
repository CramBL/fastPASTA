#!/usr/bin/env -S just --justfile

# Utility to prompt user for yes/no confirmation

# Prompts the user for yes/no input
# Returns 0 if yes, 1 if no
[private]
prompt PROMPT SUFFIX=" (y/n)":
    #!/usr/bin/env bash
    printf "%b%b%b\n" "\x1b[1;33m" "{{PROMPT}}{{SUFFIX}}" "\x1b[0m"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])+$ ]]; then
        exit 0
    else
        exit 1
    fi

# Example usage

## Note: The PROMPT variable is set in the top-level justfile

### Bash recipe
#
# my-simple-prompt:
#   #!/usr/bin/env bash
#   if {{PROMPT}} "Do you want to continue?"; then
#       do-something
#   else
#       do-something-else
#   fi

### Regular just recipe
#
# (Note: the newline escapes and semi-colons are required)
#
# my-simple-prompt:
#     if {{PROMPT}} "Do you want to continue?"; then \
#         do-something; \
#     fi
