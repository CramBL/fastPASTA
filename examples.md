# fastPASTA examples

- [fastPASTA examples](#fastpasta-examples)
- [Basic features](#basic-features)
  - [Enable custom checks](#enable-custom-checks)
  - [Dump statistics to a file](#dump-statistics-to-a-file)
  - [Use a statistics file to verify a match with the analyzed data](#use-a-statistics-file-to-verify-a-match-with-the-analyzed-data)
- [Basic scenarios](#basic-scenarios)
  - [Run all default checks down to the ITS Stave level](#run-all-default-checks-down-to-the-its-stave-level)
    - [Scenario](#scenario)
    - [Command](#command)
    - [Variations](#variations)
      - [I want to check the chip order](#i-want-to-check-the-chip-order)
  - [Inspect data at the memory position where an error was reported](#inspect-data-at-the-memory-position-where-an-error-was-reported)
    - [Scenario](#scenario-1)
    - [Command](#command-1)
    - [Variations](#variations-1)
      - [1. View ITS payload around a memory position](#1-view-its-payload-around-a-memory-position)
      - [2. View ITS payload and ALPIDE data around a memory position](#2-view-its-payload-and-alpide-data-around-a-memory-position)
- [Advanced scenarios](#advanced-scenarios)
  - [Get summaries and errors/warnings of all files in a directory arranged in a single file](#get-summaries-and-errorswarnings-of-all-files-in-a-directory-arranged-in-a-single-file)
    - [Scenario](#scenario-2)
    - [Command](#command-2)
    - [Variations](#variations-2)
      - [1. I don't want warnings, only errors](#1-i-dont-want-warnings-only-errors)
      - [2. I want to view my.log in a text viewer but ANSI escape codes ruins the summary layout.](#2-i-want-to-view-mylog-in-a-text-viewer-but-ansi-escape-codes-ruins-the-summary-layout)
      - [3. No summary report in the file, just errors/warnings](#3-no-summary-report-in-the-file-just-errorswarnings)
  - [Use the analysis of one raw data file as a golden reference for analysis of other files](#use-the-analysis-of-one-raw-data-file-as-a-golden-reference-for-analysis-of-other-files)
    - [Scenario](#scenario-3)
    - [Command](#command-3)

# Basic features
## Enable custom checks
<details>
<summary>
Click to expand
</summary>
All the checks performed with the various commands are the default checks that should always be true for the given system.

To enable checks that depend on the system configuration, you can supply a custom checks configuration file in the [TOML](https://toml.io/en/) format.
First generate the template
```shell
fastpasta --generate-checks-toml
```
Your current working directory now contains a `custom_checks.toml` file that lists all the custom checks you can enable. The custom checks follow the pattern:
- `# description`
- `# example`
- `#commented out value`
<details>
<summary>
Click to see example `custom_checks.toml`
</summary>

```toml
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
#cdps = None [ u32 ] # (Uncomment and set to enable)

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
#triggers_pht = None [ u32 ] # (Uncomment and set to enable)

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of lists of chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14], [1, 2, 3, 4, 5, 6, 7]]
#chip_orders_ob = None [ Vec < Vec < u8 > > ] # (Uncomment and set to enable)

# Number of chips expected in the data from Outer Barrel (ML/OL)
# Example: 7
#chip_count_ob = None [ u8 ] # (Uncomment and set to enable)

# The RDH version expected in the data
# Example: 7
#rdh_version = None [ u8 ] # (Uncomment and set to enable)
```
</details>
<br>

Then edit the `custom_checks.toml` to enable the checks you want and run fastpasta with the `--checks-toml` option e.g.
```shell
fastpasta datafile.raw check all its-stave --checks-toml my_custom_checks.toml
```
<details>
<summary>
Click to see example `custom_checks.toml` with enabled checks
</summary>

```toml
# Number of CRU Data Packets expected in the data
# Example: 20, 500532
cdps = 20 # Check that the data contains exactly 20 CDPs

# Number of Physics (PhT) Triggers expected in the data
# Example: 0, 10
triggers_pht = 0 # Error if the data even contains one Physics Trigger anywhere

# Legal Chip ordering for Outer Barrel (ML/OL). Needs to be a list of lists of chip IDs
# Example: [[0, 1, 2, 3, 4, 5, 6], [8, 9, 10, 11, 12, 13, 14], [1, 2, 3, 4, 5, 6, 7]]
chip_orders_ob = [[0, 1, 3, 7]] # Only the ordering 0, 1, 3, 7 is legal now, all other orderings will generate an error

# Number of chips expected in the data from Outer Barrel (ML/OL)
# Example: 7
chip_count_ob = 7 # Enabled

# The RDH version expected in the data
# Example: 7
rdh_version = 7 # Enabled
```
</details>
</details>

## Dump statistics to a file
<details>
<summary>
Click to expand
</summary>

By using `--output-stats <file_name>` in combination with `--stats-format <JSON/TOML>` all collected stats will be written to `file_name` at the end of analysis. e.g.
```shell
fastpasta MYDATAFILE.raw check sanity --output-stats mystats.json --stats-format json
```
TOML is also supported, and is usually much more readable than JSON.

</details>

## Use a statistics file to verify a match with the analyzed data
<details>
<summary>
Click to expand
</summary>

If you generated a stat dump (see [Dump statistics to a file](#dump-statistics-to-a-file)) the stat dump can be used as an input to check against all the analyzed data. e.g.
```shell
fastpasta MYDATAFILE.raw check sanity --input-stats-file mystats.json
```
>Note: the input stats file extension has to match the format in the file

An error is displayed for each mismatching value in the input stats file and the stats collected during analysis.

</details>

<br><br>

# Basic scenarios

## Run all default checks down to the ITS Stave level
<details>
<summary>
Click to expand
</summary>

### Scenario
- I have `MYBIN.raw` data file

### Command
```shell
fastpasta MYBIN.raw check all its-stave
```
<details>
<summary>Show 1 variation of this scenario</summary>

### Variations

#### I want to check the chip order
This can be achieved by using the `--checks-toml` option. See the [Enable custom checks](#enable-custom-checks) section for how to generate and set it. Then set the `chip_orders_ob` field and supply the `checks_toml` file e.g.
```shell
fastpasta MYBIN.raw check all its-stave --checks-toml mychecks.toml
```


</details>
</details>

## Inspect data at the memory position where an error was reported
<details>
<summary>
Click to expand
</summary>

### Scenario
- I have analyzed `MYBIN.raw` with `check all its-stave` and found errors in an **RDH** at memory position `0x12BEEF`
- I want to inspect the **RDH**s around that error position

### Command
```shell
# Generate the RDH view and pipe it to less
fastpasta view rdh | less
# Skip to the target RDH's memory position  by typing '/12BEEF`
# (forward slash -> memory position -> enter)
```

<details>

<summary>Show 2 variations of this scenario</summary>

### Variations
#### 1. View ITS payload around a memory position
```shell
# Generate the ITS payload view and pipe it to less
fastpasta view its-readout-frames | less
# Skip to the target RDH's memory position  by typing '/12BEEF`
```

#### 2. View ITS payload and ALPIDE data around a memory position
```shell
# Generate the ITS payload with lane data view and pipe it to less
fastpasta view its-readout-frames-data | less
# Skip to the target RDH's memory position  by typing '/12BEEF`
```
</details>
</details>


<br><br>

# Advanced scenarios

## Get summaries and errors/warnings of all files in a directory arranged in a single file
<details>
<summary>
Click to expand
</summary>

### Scenario
- I have `MY_DIRECTORY` with raw ITS readout data files with the `.raw` extension.
- I want to check all ITS data but not ALPIDE data.
- Each summary should be delimited by two newlines `--- {filename} ---` and then another newline,
- Everything should be written to `my.log`.
### Command
```shell
find MY_DIRECTORY -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its --verbosity 1 {} >> my.log 2>&1' {} \;
```
<details>

<summary>Show 3 variations of this scenario</summary>

### Variations

#### 1. I don't want warnings, only errors

Change `--verbosity 1` to `--verbosity 0`

#### 2. I want to view my.log in a text viewer but ANSI escape codes ruins the summary layout.

Use an ANSI to HTML converter like [aha](https://github.com/theZiz/aha) (available through `apt` and `dnf`) or [ansi2html](https://pypi.org/project/ansi2html/) (`pip`).
Then alter the command to pipe to the converter e.g. with `ansi2html`:
```shell
find MY_DIRECTORY -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its-stave --verbosity 1 {} 2>&1 | ansi2html >> my.log.html ' {} \;
```
It can now be viewed in any browser.
> `aha --black` gives the same result as `ansi2html`.

#### 3. No summary report in the file, just errors/warnings
Redirect `stderr` to my.log by removing `2>&1` and put `2` in front of the file appending `>>` i.e.
```shell
find MY_DIRECTORY -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its-stave --verbosity 1 {} 2>> my.log' {} \;
```
This will instead print the summaries to the terminal (stdout).

If you completely want to ignore the report summaries, different platforms have a way to mute stdout such as `/dev/null` on Unix-like. Below command is platform independent and just redirects stdout to `ignore.txt` (truncating).
```shell
find MY_DIRECTORY -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its-stave --verbosity 1 {} 2>> my.log' {} > ignore.txt \;
```
</details>
</details>

## Use the analysis of one raw data file as a golden reference for analysis of other files
<details>
<summary>
Click to expand
</summary>

### Scenario
- I have `MYGOLDENFILE.raw` and I want to verify that `MYOTHERFILE.raw` is similar down to the stave level

### Command
```shell
# Generate the golden reference
fastpasta MYGOLDENFILE.raw check all its-stave --output-stats myGoldenStats.json --stats-format json
```
```shell
# Use it to check against the other file
fastpasta MYOTHERFILE.raw check all its-stave --input-stats-file myGoldenStats.json
```
For each mismatching statistics, an error will be displayed. TOML format is also supported which is usually much more readable than JSON.
</details>
