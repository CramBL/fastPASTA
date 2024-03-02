[private]
prompt-install INSTALL_CMD:
    #!/usr/bin/env bash
    if {{PROMPT}} "Install with: {{ INSTALL_CMD }}?"; then
        eval "{{ INSTALL_CMD }}"
    fi

# Check that the nightly toolchain is installed
[private]
dep-nightly-toolchain TXT_IF_NOT_INSTALLED:
    #!/usr/bin/env bash
    if [[ $(rustup toolchain list | grep nightly --count ) -eq 0 ]]; then
        {{PRINT}} yellow "{{ TXT_IF_NOT_INSTALLED }}"
        just prompt-install "rustup toolchain install nightly"
    fi

[private]
dep-nightly-llvm-tools TXT_IF_NOT_INSTALLED:
    #!/usr/bin/env bash
    if [[ $(rustup +nightly component list | grep "llvm-tools.*installed" --count) -eq 0 ]]; then
        {{PRINT}} yellow "{{ TXT_IF_NOT_INSTALLED }}"
        just prompt-install "rustup +nightly component add llvm-tools-preview"
    fi

[private]
dep-nightly-rust-src TXT_IF_NOT_INSTALLED:
    #!/usr/bin/env bash
    if [[ $(rustup +nightly component list | grep "rust-src.*installed" --count) -eq 0 ]]; then
        {{PRINT}} yellow "{{ TXT_IF_NOT_INSTALLED }}"
        just prompt-install "rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu"
    fi