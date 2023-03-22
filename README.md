
# fastPASTA
[![pipeline status](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/pipeline.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![coverage report](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/coverage.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![Latest Release](https://gitlab.cern.ch/mkonig/fastpasta/-/badges/release.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/releases)

## fast Protocol Analysis Scanner Tool for ALICE
fastPASTA uses [Semantic Versioning](https://semver.org/).

For extensive documentation of public facing source code, invoke ```cargo doc --open```

## Purpose

To parse CRU Data Packets for protocol violations and report any errors

## To start using fastPASTA, build the binary with `cargo build -r` and find it in ./target/release/fastpasta.
### See help, including examples of use

```shell
$ ./fastpasta -h
```

### Examples
1. Read from file -> filter by link 0 -> validate with all checks enabled
```shell
$ ./fastpasta input.raw --filter-link 0 check all
```
2. Read decompressed data from stdin -> filter link 3 -> see a formatted view of RDHs
```shell
$ lz4 -d input.raw -c | ./fastpasta --filter-link 3 | ./fastpasta view rdh
         ^^^^                      ^^^^                       ^^^^
        INPUT       --->          FILTER          --->        VIEW
```

Piping is often optional and avoiding it will improve performance. e.g. the following is equivalent to the previous example, but saves significant IO overhead, by using one less pipe.
```shell
$ lz4 -d input.raw -c | ./fastpasta --filter-link 3 view rdh
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
