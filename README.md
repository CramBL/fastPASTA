
# fastPASTA
[![pipeline status](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/pipeline.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![coverage report](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/coverage.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![Latest Release](https://gitlab.cern.ch/mkonig/fastpasta/-/badges/release.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/releases)

## fast Protocol Analysis Scanner Tool for ALICE
fastPASTA uses [Semantic Versioning](https://semver.org/).

For extensive documentation of public facing source code, invoke ```cargo doc --open```

For an exhaustive list of the data verification done via the `check` subcommand, see [list of checks](doc/checks_list.md).

## Purpose

To verify or view curated content of the scanned raw binary data from ALICE.

# Table of Contents
- [fastPASTA](#fastpasta)
  - [fast Protocol Analysis Scanner Tool for ALICE](#fast-protocol-analysis-scanner-tool-for-alice)
  - [Purpose](#purpose)
- [Table of Contents](#table-of-contents)
- [Quickstart](#quickstart)
  - [Prerequisite](#prerequisite)
  - [Build the binary with `cargo build -r` and find it in /target/release/fastpasta](#build-the-binary-with-cargo-build--r-and-find-it-in-targetreleasefastpasta)
  - [See help, including examples of use](#see-help-including-examples-of-use)
- [Examples of use](#examples-of-use)
    - [Read from stdin -\> filter link -\> view RDHs](#read-from-stdin---filter-link---view-rdhs)
    - [Read from file -\> filter by link -\> validate](#read-from-file---filter-by-link---validate)
    - [Read from file -\> view HBFs with `less`](#read-from-file---view-hbfs-with-less)
- [Error messages](#error-messages)
    - [Messages are formatted as follows:](#messages-are-formatted-as-follows)
    - [Example of failed RDH sanity check](#example-of-failed-rdh-sanity-check)
    - [Error codes are not unique](#error-codes-are-not-unique)
  - [Verbosity levels](#verbosity-levels)
- [License](#license)
- [Project status](#project-status)
- [Benchmarks and comparisons](#benchmarks-and-comparisons)
    - [Verifying all RDHs in 3.4GB file](#verifying-all-rdhs-in-34gb-file)
    - [Verifying all RDHs in 500MB file](#verifying-all-rdhs-in-500mb-file)
- [Need more performance?](#need-more-performance)
    - [Background](#background)
    - [To install the nightly toolchain (and check your installation)](#to-install-the-nightly-toolchain-and-check-your-installation)
    - [Compile the optimized `release-nightly` experimental build profile](#compile-the-optimized-release-nightly-experimental-build-profile)
    - [Path to binary: `/target/release-nightly/fastpasta`](#path-to-binary-targetrelease-nightlyfastpasta)

# Quickstart
## Prerequisite
The [rust toolchain](https://www.rust-lang.org/tools/install) is required to compile the binary. On macOS, Linux or other Unix-like OS simply run
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
and follow the on-screen instructions.
## Build the binary with `cargo build -r` and find it in /target/release/fastpasta
## See help, including examples of use

```shell
$ ./fastpasta -h
```

# Examples of use
### Read from stdin -> filter link -> view RDHs
```shell

$ lz4 -d input.raw -c | ./fastpasta --filter-link 3 | ./fastpasta view rdh
#        ^^^^                      ^^^^                       ^^^^
#       INPUT       --->          FILTER          --->        VIEW
# Decompressing with `lz4`
```

Piping is often optional and avoiding it will improve performance. e.g. the following is equivalent to the previous example, but saves significant IO overhead, by using one less pipe.
```shell
$ lz4 -d input.raw -c | ./fastpasta --filter-link 3 view rdh
```
### Read from file -> filter by link -> validate
```shell
# Enable all generic checks: `sanity` (stateless) AND `running` (stateful)
$ ./fastpasta input.raw --filter-link 0 check all

# Same as above but only enable `sanity` checks
$ ./fastpasta input.raw check sanity -f 0

# Enable all `sanity` and `running` checks and include checks applicable to `ITS` only
$ ./fastpasta input.raw check all ITS --filter-link 0

# Filter link 3 and check `sanity` include sanity checks specific to ITS
$ ./fastpasta input.raw -f 3 check sanity its # target `its` is case-insensitive
```

### Read from file -> view HBFs with `less`
```shell
# Generate HBF view
$ ./fastpasta input.raw view hbf | less
# View only HBFs from link #3
$ ./fastpasta input.raw view hbf -f 3 | less
```

# Error messages
### Messages are formatted as follows:

```shell
MEMORY_OFFSET: [ERROR_CODE] ERROR_MESSAGE
```
### Example of failed RDH sanity check
```shell
0xE450FFD: [E10] RDH sanity check failed: data_format = 255
```

### Error codes are not unique
But they signify categories of errors.
E.g. all RDH sanity checks have the same error code, but the error message will specify which field failed.
The following is a list of error codes and their meaning, `x` is a placeholder for any number 0-9.
* [Ex0] - Sanity check
* [E1x] - RDH
* [E3x] - IHW
* [E4x] - TDH
* [E5x] - TDT
* [E6x] - DDW0
* [E7x] - Data word (Even number: IB, Odd number: OB) E70 is sanity check for both IB/OB.
* [E8x] - CDW

## Verbosity levels
- 0: Errors
- 1: Errors and warnings
- 2: Errors, warnings and info
- 3: Errors, warnings, info and debug
- 4: Errors, warnings, info, debug and trace

# License
Apache 2.0 or MIT at your option.

# Project status
Under development

# Benchmarks and comparisons
In the table below `fastPASTA` is compared with `rawdata-parser` in verifying each RDH on a 3.4GB large file with raw ITS data. `decode.py` does not support the new data format using UL, so it is run on a much smaller file 500MB file, but with similar data contents (except for dataformat), and the execution time is multiplied by 6. Hyperfine is used for benchmarking, but `rawdata-parser` is not compatible with hyperfine, and is therefor measured with the `time` command.
### Verifying all RDHs in 3.4GB file
| Tool | Command | Mean [s] | Min [s] | Max [s] |
|:---|:---|---:|---:|---:|
|fastPASTA| `./fastpasta data_ols_its-ul-v0.5_3.4GB check all` | 1.058 ± 0.021 | 1.031 | 1.101 |
|rawdata-parser| `racket rawdata-parser.rkt --skip-packet-counter-checks data_ols_its-ul-v0.5_3.4GB` | 5.201 | N/A | N/A |
|decode.py| `python3 decode.py -i 20522 -i 20778 -f data_ols_no_ul.raw --skip_data` | 79.26 ± 1.57 | 76.64 | 82.236 |

### Verifying all RDHs in 500MB file
| Tool | Command | Mean [s] | Min [s] | Max [s] |
|:---|:---|---:|---:|---:|
|fastPASTA| `./fastpasta data_ols_no_ul.raw check all` | 0.123 ± 0.023 | 0.119 | 0.126 |
|rawdata-parser| `racket rawdata-parser.rkt --skip-packet-counter-checks data_ols_no_ul.raw ` | 1.580 | N/A | N/A |
|decode.py| `python3 decode.py -i 20522 -i 20778 -f data_ols_no_ul.raw --skip_data` | 13.151 ± 0.217 | 12.870 | 13.634 |


# Need more performance?
The primary release profile of fastPASTA is already very fast, but if you absolutely need 10-20% more speed, a faster build profile exists that utilizes the experimental `rust nightly` toolchain.

### Background
The rust compiler `rustc` does not yet provide access to all the features that its backend `LLVM` has. But the experimental `nightly` rust toolchain allows passing flags directly to `LLVM`. fastPASTA includes configuration for a build profile `release-nightly` which utilizes `LLVM` to achieve more speed at the cost of compilation time and binary size. As of this writing, the stable channel of Rust does not have a way to pass compiler flags to the LLVM backend. The increased speed is mainly achieved through configuring a higher threshold for inlining, which will increase speed but also compilation time and binary size, and most crucially, cache pressure. The performance impact will be highly dependent on the machine fastPASTA runs on. Better/more CPU cache will lead to a higher performance gain. With >1 GB individual link data, the performance on one particular CERN machine running CentOS Stream 8, as measured by [`hyperfine`](https://github.com/sharkdp/hyperfine) increased by ~17%.

### To install the nightly toolchain (and check your installation)
```shell
$ rustup toolchain install nightly
$ rustup run nightly rustc --version
```
### Compile the optimized `release-nightly` experimental build profile
```shell
$ cargo +nightly build --profile release-nightly
```
### Path to binary: `/target/release-nightly/fastpasta`
