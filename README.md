
# fastPASTA
[![pipeline status](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/pipeline.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
 [![Latest Release](https://gitlab.cern.ch/mkonig/fastpasta/-/badges/release.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/releases)
 [![coverage report](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/coverage.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
## fast Protocol Analysis Scanner Tool for ALICE

For extensive documentation of source code, invoke ```cargo doc --open```

## Purpose

To parse CRU Data Packets for protocol violations and report any errors

## To start using fastPASTA, build the binary with `cargo build -r` and find it in ../target/release/fastpasta, or download the latest release from the [releases page](https://gitlab.cern.ch/mkonig/fastpasta/-/releases).
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


## Roadmap
- [x] Parse RDH + Payload
- [x] Parse HBF with multiple CDPs
- [x] Parse CDP in UL flavor 1
- [x] Parse CDP in UL flavor 0
- [x] Validate CDPs are in am HBF pattern (First is page counter 0, stop bit 0, page counter then increments and only last CDP has stop bit 1)
- [x] Validate all 80-bit GBT words with sanity checks on IDs
  - [x] Status Words
  - [x] Data words
- [x] Validate CDPs with a sanity check on the structure adhering to the CRU protocol
- [x] Filter data by GBT link
- [x] Validate the CDP payload is following the ITS protocol
- [ ] Filter data from individual ALPIDE chips
- [ ] Advanced protocol checks in split HBFs (multiple CDPs)
- [ ] Validate ALPIDE payload
- [ ] Convert UL flavor 0 to UL flavor 1 and vice versa
- [ ] Detect when event data has been dropped and validate the subsequent diagnostic flags in the HBF

### CDP payload checks (cross checks on RDH)
The validation follows the FSM below. For each state, sanity checks are performed on the GBT words before transition.
Certain transitions are ambigious (marked by yellow notes), these are resolved based on the ID of the next received GBT word.
![CDP FSM for validation](doc/CDP_payload_StateMachine%20(continuous%20mode).png)

## License
Apache or MIT at your option.

## Project status
Under development
