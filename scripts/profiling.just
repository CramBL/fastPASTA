
# This profiling recipe uses flamegraph: https://github.com/flamegraph-rs/flamegraph?tab=readme-ov-file#systems-performance-work-guided-by-flamegraphs

TMP_DATA_PATH := "target/tmp_data.raw"

# Generate a flamegraph for profiling fastpasta
[private]
gen-flamegraph DATA_FILE ARGS DATA_SIZE="500" $CARGO_PROFILE_RELEASE_DEBUG="true": flamegraph-deps && (open-in-browser "target/flamegraph.svg")
    @{{PRINT}} cyan "Building in release mode with debug info CARGO_PROFILE_RELEASE_DEBUG=${CARGO_PROFILE_RELEASE_DEBUG}"
    cargo build --release
    @{{PRINT}} cyan "Growing {{ DATA_FILE }} to {{ DATA_SIZE }} at {{ TMP_DATA_PATH }}"
    just bin-mult-grow "{{ DATA_FILE }}" "{{ DATA_SIZE }}" "{{ TMP_DATA_PATH }}"
    @{{PRINT}} cyan "Running flamegraph on: fastpasta {{ TMP_DATA_PATH }} {{ ARGS }}"
    just run-flamegraph "target/flamegraph.svg" "target/release/fastpasta" "{{ TMP_DATA_PATH }}" "{{ ARGS }}"
    @{{PRINT}} cyan "Done. Cleaning up by removing {{ TMP_DATA_PATH }}"
    rm -f "{{ TMP_DATA_PATH }}"

[private]
run-flamegraph SVG BIN DATA ARGS:
    #!/usr/bin/env bash
    start=${SECONDS}
    flamegraph --output "{{ SVG }}" -- "{{ BIN }}" "{{ DATA }}" {{ ARGS }}
    {{PRINT}} green "Flamegraph finished in $(( ${SECONDS} - start )) seconds"

# Run perf stat on fastpasta
[private]
perf-profile DATA_FILE SIZE_MB ARGS REPEAT="3": perf-deps
    cargo build --release
    just bin-mult-grow "{{ DATA_FILE }}" "{{ SIZE_MB }}" "target/tmp_raw_data.raw"
    perf stat --repeat "{{ REPEAT }}" -B target/release/fastpasta "target/tmp_raw_data.raw" {{ ARGS }}
    rm -f "target/tmp_raw_data.raw"


[private]
ci-perf-profile: ci-perf-deps
    cargo build --release
    just bin-mult-grow "tests/test-data/12_links_2hbf.raw" "500" "target/tmp_raw_data.raw"
    perf stat -B target/release/fastpasta "target/tmp_raw_data.raw" --mute-errors check sanity
    perf stat -B target/release/fastpasta "target/tmp_raw_data.raw" --mute-errors check all
    perf stat -B target/release/fastpasta "target/tmp_raw_data.raw" --mute-errors check all its
    perf stat -B target/release/fastpasta "target/tmp_raw_data.raw" --mute-errors check all its-stave
    rm -f "target/tmp_raw_data.raw"

[private]
ci-perf-deps:
    #!/usr/bin/env bash
    cargo install binmult
    if ! which perf; then
        apt-get update -qq
        apt-get --assume-yes install linux-tools-common linux-tools-generic linux-tools-`uname -r`
    fi

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
