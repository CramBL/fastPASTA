COV_ENV_RUST_BACKTRACE := "FULL"
COV_ENV_CARGO_INCREMENTAL := "0"
COV_ENV_RUSTFLAGS := "-Cinstrument-coverage -Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
COV_ENV_RUSTDOCFLAGS := "-Cpanic=abort"
COV_ENV_LLVM_PROFILE_FILE := "target/coverage/%p-%m.profraw"

run-test-coverage ARG="" COV_DATA_DIR="target/coverage": coverage-dependencies 
    #!/usr/bin/env bash
    export RUSTUP_TOOLCHAIN="nightly"
    export RUST_BACKTRACE="{{ COV_ENV_RUST_BACKTRACE }}"
    export CARGO_INCREMENTAL="{{ COV_ENV_CARGO_INCREMENTAL }}"
    export RUSTFLAGS="{{ COV_ENV_RUSTFLAGS }}"
    export RUSTDOCFLAGS="{{ COV_ENV_RUSTDOCFLAGS }}"
    export LLVM_PROFILE_FILE="{{ COV_ENV_LLVM_PROFILE_FILE }}"
    rm -f "alice_protocol_reader/{{COV_DATA_DIR}}/"*.profraw
    rm -f "fastpasta/{{COV_DATA_DIR}}/"*.profraw
    rm -f "{{COV_DATA_DIR}}/"*.profraw
    cargo build
    cargo test
    grcov . \
            -s . \
            --binary-path ./target/debug/ \
            --output-types html,cobertura \
            --branch \
            --ignore-not-existing \
            --ignore "$HOME/.cargo/**" \
            -o {{ COV_DATA_DIR }}/ \
            --keep-only 'fastpasta/src/*' \
            --keep-only 'alice_protocol_reader/src/*' \
            --excl-start '\#\[cfg\(test\)\]'
    xmllint --xpath "concat('Coverage: ', 100 * string(//coverage/@line-rate), '%')" {{ COV_DATA_DIR }}/cobertura.xml
    if [[ "{{ ARG }}" =~ ^.*([oO][pP][eE]|[nN])+$ ]]; then
        just open-coverage-report "{{ COV_DATA_DIR }}/html/index.html"
    fi

# Attempts to open the code coverage report at PATH with Firefox or Google Chrome
[private]
open-coverage-report PATH="target/coverage/html/index.html":
    #!/usr/bin/env bash
    if [ ! -f "{{ PATH }}" ]; then
        {{PRINT}} red "No coverage report at {{ PATH }}"
        {{PRINT}} cyan "Run the test coverage recipe and try again"
        exit 1
    fi
    if which firefox > /dev/null; then 
        firefox "{{ PATH }}"
    fi
    if which google-chrome > /dev/null; then
        google-chrome "{{ PATH }}"
    fi

# Check for dependencies of coverage collection
[private]
coverage-dependencies:
    #!/usr/bin/env bash
    if [[ $(rustup toolchain list | grep nightly --count ) -eq 0 ]]; then
        {{PRINT}} yellow "Coverage requires the nightly toolchain"
        just prompt-install "rustup toolchain install nightly"
    fi
    if [[ $(rustup +nightly component list | grep "llvm-tools.*installed" --count) -eq 0 ]]; then
        {{PRINT}} yellow "Coverage requires llvm-tools-preview" 
        just prompt-install "rustup +nightly component add llvm-tools-preview"
    fi
    if ! which grcov >/dev/null; then
        {{PRINT}} yellow "Coverage requires grcov"
        just prompt-install "cargo install grcov"
    fi
    if ! which xmllint >/dev/null; then
        {{PRINT}} red "xmllint is required for generating the coverage report"
        {{PRINT}} cyan "xmllint is included in the libxml2-utils, install it with your systems package manager to continue"
    fi