# fastPASTA
## fast Protocol Analysis Scanner Tool for ALICE

For extensive documentation, invoke ```cargo doc --open```

`fastPASTA` follows the CLI guidelines for design https://clig.dev/.

`fastPASTA` only uses stderr to print information for the user, and stdout is reserved for the output of processed data.

## Purpose

To parse CRU Data Packets for protocol violations and report any errors

## To start using fastPASTA, simply execute fastpasta from the compiled binary in ../target/debug/fastpasta
### See help

```shell
$ ./target/debug/fastpasta -h
```


### Parse a file with sanity checks on RDH and ITS Status words, only take only link 2, only print errors and warnings, tolerate up to 10 errors before ending processing, and save it to test.raw
```shell
$ ./target/release/fastpasta --sanity-checks --filter-link 2 --verbosity 1 --tolerate-max-errors 10 ../fastpasta_test_files/file_to_process --output test.raw
```
Equivelant to the following, using the short form of the arguments:
```shell
$ ./target/release/fastpasta -s -f 2 -v 1 -e 10 ../fastpasta_test_files/file_to_process -o test.raw
```
### Verbosity levels
- 0: Errors
- 1: Errors and warnings
- 2: Errors, warnings and info
- 3: Errors, warnings, info and debug
- 4: Errors, warnings, info, debug and trace


## Parsing the printouts to stderror with `grep` or `sed`
If you want to parse the output messages from fastPASTA, it is recommend to redirect the stderr to stdout and then parsing it with e.g. grep.

**Example: Get all information about parsed RDHs from link 0**
RDH info printouts are multi-line so match RDH and include the next 2 lines in the match.
```shell
$ ./fastpasta -s -f 0 -v 3 ../fastpasta_test_files/file_to_process 2>&1 | grep -A 2 RDH
```
Include also all the `TDH`'s, now using `sed`
```shell
$ ./fastpasta -s -f 0 -v 3 ../fastpasta_test_files/file_to_process 2>&1 | sed -n -e '/RDH/{N;N;p}' -e '/TDH/ p'
```
## Roadmap
- [x] Parse RDH + Payload
- [x] Parse HBF with multiple CDPs
- [x] Parse CDP in UL flavor 1
- [x] Parse CDP in UL flavor 0
- [x] Validate CDPs are in am HBF pattern (First is page counter 0, stop bit 0, page counter then increments and only last CDP has stop bit 1)
- [ ] Validate all 80-bit GBT words with sanity checks on IDs
  - [x] Status Words
  - [ ] Data words
- [x] Validate CDPs with a sanity check on the structure adhering to the CRU protocol
- [x] Filter data by GBT link
- [ ] Validate the CDP payload is following the ITS protocol
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
