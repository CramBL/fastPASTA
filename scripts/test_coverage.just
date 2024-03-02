# Run tests and collect instrumentation based coverage

[private]
run-test-coverage   \
        ARG="" \
        COV_DATA_DIR="target/coverage" \
        $RUSTUP_TOOLCHAIN="nightly" \
        $RUST_BACKTRACE="FULL" \
        $CARGO_INCREMENTAL="0" \
        $RUSTFLAGS="-Cinstrument-coverage -Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" \
        $RUSTDOCFLAGS="-Cpanic=abort" \
        $LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw" \
        : coverage-dependencies
    @{{PRINT}} cyan "RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN}\nRUST_BACKTRACE=${RUST_BACKTRACE}\nCARGO_INCREMENTAL=${CARGO_INCREMENTAL}\nRUSTFLAGS=${RUSTFLAGS}\nRUSTDOCFLAGS=${RUSTDOCFLAGS}\nLLVM_PROFILE_FILE=${LLVM_PROFILE_FILE}"
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
coverage-dependencies: \
(dep-nightly-toolchain "Coverage requires the nightly toolchain") \
(dep-nightly-llvm-tools "Coverage requires llvm-tools-preview")
    #!/usr/bin/env bash
    if ! which grcov >/dev/null; then
        {{PRINT}} yellow "Coverage requires grcov"
        just prompt-install "cargo install grcov"
    fi
    if ! which xmllint >/dev/null; then
        {{PRINT}} red "xmllint is required for generating the coverage report"
        {{PRINT}} cyan "xmllint is included in the libxml2-utils, install it with your systems package manager to continue"
    fi
