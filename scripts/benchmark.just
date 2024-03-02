
# Run benchmarks $cmds indicates which commands are included in the benchmarks
bench $cmds=" check sanity; check all; check all its; check all its-stave":
    ./tests/regression/regression_performance.sh

# Benchmark with the default values in the script
[private]
bench-default:
    ./tests/regression/regression_performance.sh

# Extended benchmark with larger amounts of data and more iterations
[private]
bench-extended $EXTENDED="True":
    ./tests/regression/regression_performance.sh

