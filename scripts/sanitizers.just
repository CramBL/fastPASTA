RUSTUP_TOOLCHAIN :="nightly"
RUST_BACKTRACE := "FULL"

ASAN_ENV_ASAN_OPTIONS := "detect_odr_violation=0:detect_leaks=0"
ASAN_ENV_RUSTFLAGS := "-Z sanitizer=address"

LSAN_ENV_LSAN_OPTIONS := "suppressions=lsan-suppressions.txt"
LSAN_ENV_RUSTFLAGS := "-Z sanitizer=leak"

TSAN_ENV_TSAN_OPTIONS := ""
TSAN_ENV_RUSTFLAGS := "-Z sanitizer=thread"

MSAN_ENV_MSAN_OPTIONS := ""
MSAN_ENV_RUSTFLAGS := "-Z sanitizer=memory -Zsanitizer-memory-track-origins"

SAN_BUILD_FLAGS := "-Z build-std --target x86_64-unknown-linux-gnu"

# Run the full suite of sanitizers
test-sanitizers *ARGS: (sanitizer-build RUSTUP_TOOLCHAIN RUST_BACKTRACE ARGS)
    @{{PRINT}} green "=== STARTING ASAN TEST ==="
    just _run-test-asan "{{RUSTUP_TOOLCHAIN}}" "{{RUST_BACKTRACE}}" "{{ASAN_ENV_RUSTFLAGS}}" "{{ASAN_ENV_ASAN_OPTIONS}}" {{ ARGS }}
    @{{PRINT}} green "=== STARTING LSAN TEST ==="
    just _run-test-lsan "{{RUSTUP_TOOLCHAIN}}" "{{RUST_BACKTRACE}}" "{{LSAN_ENV_RUSTFLAGS}}" "{{LSAN_ENV_LSAN_OPTIONS}}" {{ ARGS }}
    @{{PRINT}} yellow "Skipping buggy Thread and Memory sanitizers"
    #{{PRINT}} green "=== STARTING TSAN TEST ==="
    #just _run-test-tsan "{{RUSTUP_TOOLCHAIN}}" "{{RUST_BACKTRACE}}" "{{MSAN_ENV_RUSTFLAGS}}" "{{MSAN_ENV_MSAN_OPTIONS}}" {{ ARGS }}
    #{{PRINT}} green "=== STARTING MSAN TEST ==="
    #just _run-test-msan "{{RUSTUP_TOOLCHAIN}}" "{{RUST_BACKTRACE}}" "{{TSAN_ENV_RUSTFLAGS}}" "{{TSAN_ENV_TSAN_OPTIONS}}" {{ ARGS }}
    
# Build with instrumentation to prepare for tests with sanitizers
[private]
sanitizer-build $RUSTUP_TOOLCHAIN $RUST_BACKTRACE *ARGS: sanitizer-dependencies clean
    @{{PRINT}} cyan "RUST_BACKTRACE=${RUST_BACKTRACE}"
    cargo +nightly build {{ SAN_BUILD_FLAGS }} {{ ARGS }}

# Run tests with address sanitizer
test-asan *ARGS: (sanitizer-build RUSTUP_TOOLCHAIN RUST_BACKTRACE ARGS) (_run-test-asan RUSTUP_TOOLCHAIN RUST_BACKTRACE ASAN_ENV_RUSTFLAGS ASAN_ENV_ASAN_OPTIONS ARGS)
_run-test-asan $RUSTUP_TOOLCHAIN $RUST_BACKTRACE $RUSTFLAGS $ASAN_OPTIONS *ARGS:
    @{{PRINT}} cyan "RUST_BACKTRACE=${RUST_BACKTRACE}\nRUSTFLAGS=${RUSTFLAGS}\nASAN_OPTIONS=${ASAN_OPTIONS}"
    cargo +nightly test --lib --tests --all-features --target x86_64-unknown-linux-gnu {{ ARGS }}

# Run tests with leak sanitizer
test-lsan *ARGS: (sanitizer-build RUSTUP_TOOLCHAIN RUST_BACKTRACE ARGS) (_run-test-lsan RUSTUP_TOOLCHAIN RUST_BACKTRACE LSAN_ENV_RUSTFLAGS LSAN_ENV_LSAN_OPTIONS ARGS)
_run-test-lsan $RUSTUP_TOOLCHAIN $RUST_BACKTRACE $RUSTFLAGS $LSAN_OPTIONS *ARGS:    
    @{{PRINT}} cyan "RUST_BACKTRACE=${RUST_BACKTRACE}\nRUSTFLAGS=${RUSTFLAGS}\nLSAN_OPTIONS=${LSAN_OPTIONS}"
    cargo +nightly test --target x86_64-unknown-linux-gnu {{ ARGS }}

# Run tests with memory sanitizer
test-msan *ARGS: (sanitizer-build RUSTUP_TOOLCHAIN RUST_BACKTRACE ARGS) (_run-test-msan RUSTUP_TOOLCHAIN RUST_BACKTRACE MSAN_ENV_RUSTFLAGS MSAN_ENV_MSAN_OPTIONS ARGS)
_run-test-msan $RUSTUP_TOOLCHAIN $RUST_BACKTRACE $RUSTFLAGS $MSAN_OPTIONS *ARGS:
    @{{PRINT}} cyan "RUST_BACKTRACE=${RUST_BACKTRACE}\nRUSTFLAGS=${RUSTFLAGS}\nMSAN_OPTIONS=${MSAN_OPTIONS}"
    cargo +nightly test --lib --tests --all-features --target x86_64-unknown-linux-gnu {{ ARGS }}

# Run tests with thread sanitizer
test-tsan *ARGS: (sanitizer-build RUSTUP_TOOLCHAIN RUST_BACKTRACE ARGS) (_run-test-tsan RUSTUP_TOOLCHAIN RUST_BACKTRACE TSAN_ENV_RUSTFLAGS TSAN_ENV_TSAN_OPTIONS ARGS)
_run-test-tsan $RUSTUP_TOOLCHAIN $RUST_BACKTRACE $RUSTFLAGS $TSAN_OPTIONS *ARGS:
    @{{PRINT}} cyan "RUST_BACKTRACE=${RUST_BACKTRACE}\nRUSTFLAGS=${RUSTFLAGS}\nTSAN_OPTIONS=${TSAN_OPTIONS}"
    cargo +nightly test --lib --tests --all-features --target x86_64-unknown-linux-gnu {{ ARGS }}


# Check for dependencies of leak sanitizing
[private]
sanitizer-dependencies: \
(dep-nightly-toolchain "LSAN requires the nightly toolchain") \
(dep-nightly-llvm-tools "LSAN requires llvm-tools-preview") \
warn-if-no-llvm-symbolizer

[private]
warn-if-no-llvm-symbolizer:
    #!/usr/bin/env bash
    if ! which llvm-symbolizer >/dev/null; then
        {{PRINT}} yellow "WARNING: No llvm-symbolizer in PATH, sanitizer errors won't be correlated to source code"
        if [[ $(ls -l /usr/bin/llvm-symbolizer-* | wc -l) -ne 0 ]]; then
            {{PRINT}} cyan "A possible match exists in /usr/bin:"
            {{PRINT}} cyan "$(ls -l /usr/bin/llvm-symbolizer-*)"
            {{PRINT}} cyan "Creating a symbolic link at /usr/bin/llvm-symbolizer to an llvm-symbolizer binary will resolve the issue"
            {{PRINT}} green "EXAMPLE: sudo ln -s /usr/bin/llvm-symbolizer-17 /usr/bin/llvm-symbolizer"
            {{PRINT}} green "The following binary might be a match: $(ls /usr/bin/llvm-symbolizer-* | head -n 1)"
        fi
    fi
