
# fastPASTA
[![pipeline status](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/pipeline.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![coverage report](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/coverage.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![Latest Release](https://gitlab.cern.ch/mkonig/fastpasta/-/badges/release.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/releases)

## fast Protocol Analysis Scanner Tool for ALICE
fastPASTA uses [Semantic Versioning](https://semver.org/).

For extensive documentation of public facing source code, invoke ```cargo doc --open```

## Purpose

To parse CRU Data Packets for protocol violations and report any errors

## To start using fastPASTA, build the binary with `cargo build -r` and find it in ../target/release/fastpasta.
### See help, including examples of use

```shell
$ ./fastpasta -h
```

### Examples
1. Read from file -> filter by link 0 -> validate -> output to file
```shell
$ ./fastpasta input.raw --filter-link 0 --sanity-checks -o link0_output.raw
```
2. Read decompressed data from stdin -> filter link 3 & 4 -> pipe to validation checks
```shell
$ lz4 -d input.raw | ./fastpasta --filter-link 3 4 | ./fastpasta --sanity-checks
        ^^^^                   ^^^^                           ^^^^
       INPUT    --->          FILTER             --->        VALIDATE
```

### Verbosity levels
- 0: Errors
- 1: Errors and warnings
- 2: Errors, warnings and info
- 3: Errors, warnings, info and debug
- 4: Errors, warnings, info, debug and trace

## License
Apache or MIT at your option.

## Project status
Under development
