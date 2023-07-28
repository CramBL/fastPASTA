# v1.13.0 - Add ALPIDE stats (2023-07-26)
Add ALPIDE stats table when using commands that check ALPIDE data. Currently only includes stats about the readout flags of the chip trailer.

# v1.12.2 - Performance improvements (2023-07-20)
Benchmarks show >10% performance improvements over `v1.12.1` across all check commands with varying types of files.
[Example CI benchmark](https://gitlab.cern.ch/mkonig/fastpasta/-/jobs/31170333).

Benchmarks are performed with the [regression_performance.sh script](https://gitlab.cern.ch/mkonig/fastpasta/-/blob/master/tests/regression/regression_performance.sh).

### Optimizations
- Load payloads with unitialized (but allocated) memory.
- Reduce size disparity of enums by replacing dynamically sized types with boxed slices.
- Implement Copy on smaller structs (smaller or very close to usize, assume usize is >64 bits)
- Change some trivial pass-by-value to borrows
- Replace all stable sort with unstable sort.

# v1.12.0 - Unique error codes, custom checks for OB chip count (2023-07-13)
### Features
- Custom check for chip count in OB (ML/OL)
- Error codes are now unique

### Fix
- Disable spinner when `view` command is used


### Code quality
Add a CI job that performs benchmarks on the local binary vs. the latest published version and fails on >10% performance regression.

# v1.11.0 - Add ability to define custom checks through a TOML file (2023-07-06)
### Features
- Adds the `--generate-checks-toml` flag to generate a TOML file that is a template to configuring customized checks on raw data
- Adds the `--checks-toml <PATH>` option to supply a TOML file with customized checks to do on the raw data in addition to other checks

Checks for `CDP` count and `PhT trigger` count are implemented in this way. Check for valid Chip ID order for Outer Barrell is also implemented.

The generated file looks like this as of this writing:
```toml
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
#cdps = None [ u32 ] # (Uncomment and set to enable this check)

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
#triggers_pht = None [ u32 ] # (Uncomment and set to enable this check)

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
#chip_orders_ob = None [ (Vec < u8 >, Vec < u8 >) ] # (Uncomment and set to enable this check)
```

Enabling all the currently supported custom checks could look like this:
```toml
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
cdps = 20

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
triggers_pht = 0

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of two lists of 7 chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
chip_orders_ob = [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14]]
```


# v1.10.0 - Add `progress bar`, improve report formatting, add command aliases, and minor fixes (2023-06-19)
Adds a new "progress bar" that dynamically counts and displays how many HBFs are being analyzed, and continuously counts if and how many errors are detected in the data.

Adds several command aliases. Tired of writing `--filter-its-stave` and think `-s` is just too cryptic? Now there's middle grounds for several commands, e.g. write `--stave` or `--its-stave` to filter by an ITS stave.


Fixes minor issues with the report summary and improves its formatting. Small improvements to CLI argument validation.

# v1.9.0 - Check ITS lane and ALPIDE checks and new `view its-readout-frames-data` command (2023-06-14)
## Features
View formatted ITS readout frames including datawords with `view its-readout-frames-data`

## Adds The following checks to the `check all its-stave` command
### Staves & lanes
**In a readout frame**
Lanes that pass all ALPIDE checks are checked to see if their bunch counters match.
* `IB`
  * Data from 3 lanes
    * Grouped by lane ID in any of the combinations: [0, 1, 2], [3, 4, 5], or [6, 7, 8]
* `ML`
  * Data from 8 lanes
* `OL`
  * Data from 14 lanes

### ALPIDE chips

**In a readout frame**:
  * All Chip BC are identical
  * `IB`
    * Chip ID == Lane ID
  * `OB`
    * 7 Chip IDs per lane
      * Chip IDs appear in order [0-6] or [9-14]

## Other
- Add Error codes to report summary
- Add Layer/stave to RDH row in `view its-readout-frames`/`view its-readout-frames-data` commands


# v1.8.0 - Check ITS Staves in parallel - better report summary (2023-06-12)
## Features
- Add option to `--mute-errors` which will prevent displaying of error messages, something that can have significant impact on execution time in cases where data contains high amounts of errors (>10k). Useful if you just wanna see the report summary at end of execution.
- The `check all its-stave` command no longer needs the `--filter-its-stave` option, by default all staves are processed (in parallel).
- The report summary shows staves in red if any errors were detected in data from that stave (the rest are shown in white text).
- Add `orbit_bc` to RDH row of the view generated by the `view its-readout-frames` command

## Other
- Better report summary, brighter colors and more color-coded information.
- Replace ring buffer implementation to use one which only allocates on the stack.
- Slightly more compact report summary.



# v1.7.4 - New option for custom exit code, big performance increase, minor fixes (2023-06-05)
## Features
- Add feature to set a custom exit code to use if any input data fails validation, useful for using fastpasta in CI.

## Fix
- Fix colors sticking around in the terminal after an interrupt. Now `CTRL+C` initiates a graceful shutdown, a second `CTRL+C` will immediately exit.
- Fix a corner case that caused a thread to hang.


## Other
- Significant deserialization performance increase [about ~4x](https://gitlab.cern.ch/mkonig/fastpasta/uploads/a2bd71cc8ce40b356cad88e62b4719db/rdh_deserialize_impl.png)
- Some improvements to APIs
- Small improvements to error messages.


# v1.6.10 - New checks on timestamps including ALPIDE chips, small fixes (2023-05-08)
### Features
- Add `check all its_stave` subcommand to enable checks that are valid when data is coming from a single ITS Stave. this includes parsing ALPIDE data and asserting that all bunch counters from chips in each lane in a data frame are the same.
- Add `--its-trigger-period` that can be used to specify a trigger period that all `TDH`s with internal trigger set should be checked against.
- Add `Trigger_BC` column to `view its-readout-frames` command.

### Fix:
- Edge case in ITS payload fsm that didn't accept a TDH with `no_data == 1` followed by another `TDH` or an `IHW`.
- The newly introduced bug with the changes to the report summary being printed to stdout instead of stderr The change breaks piping from e.g. a fastpasta filtering and piping to another fastpasta that is checking like: `fastpasta --filter-link 8 10_rdh.raw | fastpasta check sanity` because the report summary also gets piped. Fixed it by not printing the report when `DataOutputMode` is `Stdout`.

# v1.3.9 - new view command `view its-readout-frames` - fix another ITS payload validation edge case (2023-05-05)
Get the newest version with `cargo install fastpasta`

### Features
`view its-readout-frames` - Show RDH and status word from an ITS payload. Only checks the ID field to decide how to display the word. If the ID is invalid, instead an error message is printed showing the raw bytes of the word, and parsing continues.

### Fix
- Another case related to TDH `no_data` transitions. It is now valid to transition from a TDH `no_data=1` to an `IHW` (of course separated by an RDH). See #29 for more details.

# v1.2.8 - New filter options - fixes to validation edge cases, report formatting, strict ordering of error logs (2023-05-04)
Get the newest version with `cargo install fastpasta`

### Features
`--filter-fee` allows filtering by a FEE ID

`--filter-its-stave` allows filtering by specifying a layer/stave string such as `L2_12` for layer 2 stave 12.

### Fix
- Missing check on TDH `no_data` in one transition of the ITS payload FSM that lead to protocol tracking breaking.
- Bad formatting on the end of processing report when a very large number of FEE IDs are present in data.
- Race conditions in Error reporting would occasionally cause errors for lower memory locations to be reported later than errors in higher memory locations. Errors are now strictly printed ordered by the memory locations where they were detected.

# v1.0.5 - Small fixes - big improvements to QA and code quality (2023-05-01)
### Feature(s)

* Add detect subsystem from System ID and add it to summary report

### Fix

* Only add ITS specific information to the summary if ITS was the detected subsystem
* No longer printing unsupported ANSI sequences on windows. Instead disables colors on windows (by detecting OS environment). Shame if you emulate Bash on windows. Might be an option to enable it manually later (unlikely).
* Small improvements to some print messages

### Misc. changes

* Change report summary so it now prints to stdout instead of stderr

### Code quality/modularity:

* Add example in LinkValidator of how validation of another subsystem besides ITS could be added
* Divide some ITS specific validation and word definition into ITS modules
* Get rid of some local configs in modules, in favor of using a reference to the global config everywhere, while still avoiding dynamic dispatch
* Adds some tests and reduces the size of some functions
* Decouple config from the input scanner (now uses the trait)

# v1.0.3 - Minor fixes and big reliability improvements (2023-04-25)
Minor fixes, features, and big stability improvements, both long and short term in the form of regression tests. As part of any future bug fixes, a test that reproduces the bug will be added to the regression tests, so that the bug is never reintroduced in a future change.


- Add `Run trigger type` to the report summary.
   - This stat is collected by looking at the trigger type of the first parsed RDH which should always include the trigger type for the whole run, such as `SOT` or `SOC`.
- The ITS FSM used to track the ITS payload now issues warning and errors directly when an ID error is detected in a state where multiple GBT words are valid
- Views now issue warnings before a word that could not be determined because of an error as mentioned above, but makes a "best guess" and displays the word as such.
   - The guess is easily verified by looking at the accompanying binary representation of the word.
- Remove some stats from the "Filter stats" report when using the `filter` option. These stats were not actually detected in this mode, and as such were misleading
- Add a regression test suite
- Fix RDH not displayed in `view rdh`-mode if the following payload lead to a fatal error.
   - This fix also includes decoupling the payload from all modes that are only "interested" in RDHs. Which in turn speeds up execution significantly.


# v1.0.1 - Minor improvements to prints and user feedback (2023-04-05)
- Fix a case where error message had misaligned text.
- Now report to user when output is ignored, in the case where user sets output destination and also enables checks/views, as they are mutually exclusive.
- Minor code improvements.


# v1.0.0 - Stable release of feature complete fastPASTA (2023-03-28)
### fastPASTA leaves initial development and is now considered stable. New features can be added, but current features are staying, and the way they are used will stay the same.

### Summary of current features:
- Filter data by link ID `--filter-link <ID>` (can be used with all other features)
- Write filtered data to file `-f <ID> -o <output file>`
- Generate curated view of RDHs `view rdh`
- Generate curated view of HBFs `view hbf`
- Perform data validation:
   - Generic RDH sanity check `check sanity`
   - Specific sanity check targeting ITS (includes payload checks) `check sanity its`
   - Generic RDH checks including stateful _running_ checks: `check all`
   - Specific checks including _running_ RDH & payload checks targeting ITS `check all its` (case-insensitive)

# v0.1.1 - Fix for RDH load crash (2023-16-03)
Fixed crash occurring if an EOF is encountered while loading any RDH subword except for RDH0.

# v0.1.0 - Minimum Viable Product (MVP) (2023-16-03)
fastPASTA 0.1.0 (MVP) releases to early adopters.

Contains the following major functionalities:

* Filter CDPs by GBT Link ID
* Validate CDPs are in HBF pattern (validates `pages counter` and `stop bit`)
* Validate CDPs are adhering to CRU Protocol
* Validate CDP payload is following ITS Data Protocol (continuous mode)
* Sanity checks on all GBT IDs
* Support for Data format 0 & 2
