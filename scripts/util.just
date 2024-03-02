[private]
prompt-install INSTALL_CMD:
    #!/usr/bin/env bash
    if {{PROMPT}} "Install with: {{ INSTALL_CMD }}?"; then
        eval "{{ INSTALL_CMD }}"
    fi
