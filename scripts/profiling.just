
# This profiling recipe uses flamegraph: https://github.com/flamegraph-rs/flamegraph?tab=readme-ov-file#systems-performance-work-guided-by-flamegraphs

TMP_DATA_PATH := "target/tmp_data.raw"
TMP_DATA_SIZE_MIB := "1000"

# Generate a flamegraph for profiling fastpasta
gen-flamegraph DATA_FILE ARGS $CARGO_PROFILE_RELEASE_DEBUG="true": flamegraph-deps && (open-in-browser "target/flamegraph.svg")
    @{{PRINT}} cyan "Building in release mode with debug info CARGO_PROFILE_RELEASE_DEBUG=${CARGO_PROFILE_RELEASE_DEBUG}"
    cargo build --release
    @{{PRINT}} cyan "Growing {{ DATA_FILE }} to {{ TMP_DATA_SIZE_MIB }} at {{ TMP_DATA_PATH }}"
    just bin-mult-grow "{{ DATA_FILE }}" "{{ TMP_DATA_SIZE_MIB }}" "{{ TMP_DATA_PATH }}"
    @{{PRINT}} cyan "Running flamegraph on: fastpasta {{ TMP_DATA_PATH }} {{ ARGS }}"
    just run-flamegraph "target/flamegraph.svg" "target/release/fastpasta" "{{ TMP_DATA_PATH }}" "{{ ARGS }}"
    @{{PRINT}} cyan "Done. Cleaning up by removing {{ TMP_DATA_PATH }}"
    rm -f "{{ TMP_DATA_PATH }}"

[private]
run-flamegraph SVG BIN DATA ARGS:
    #!/usr/bin/env bash
    start=${SECONDS}
    flamegraph --output "{{ SVG }}" -- "{{ BIN }}" "{{ DATA }}" "{{ ARGS }}"
    {{PRINT}} green "Flamegraph finished in $(( ${SECONDS} - start )) seconds"

# Run perf on fastpasta
perf-profile DATA_FILE SIZE_MB ARGS: perf-deps
    cargo build --release
    just bin-mult-grow "{{ DATA_FILE }}" "{{ SIZE_MB }}" "target/tmp_raw_data.raw"
    perf stat -B target/release/fastpasta "target/tmp_raw_data.raw" {{ ARGS }}
    rm -f "target/tmp_raw_data.raw"

# Check dependencies for profiling with perf
[private]
perf-deps:
    #!/usr/bin/env bash
    if ! which perf >/dev/null; then
        {{PRINT}} yellow "perf not found, it is part of common/generic linux tools"
        {{PRINT}} yellow "Install perf and try again"
        exit 1
    fi

# Check dependencies for profiling with flamegraph
[private]
flamegraph-deps:
    #!/usr/bin/env bash
    if ! which flamegraph >/dev/null; then
        {{PRINT}} yellow "flamegraph not found"
        just prompt-install "cargo install flamegraph"
    fi
    if ! which binmult >/dev/null; then
        {{PRINT}} yellow "binmult not found, it is used to generate larger data files to make profiling feasible"
        just prompt-install "cargo install binmult"
    fi
