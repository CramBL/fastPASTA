# fastPASTA examples

- [fastPASTA examples](#fastpasta-examples)
- [Basic](#basic)
- [Advanced](#advanced)
  - [Get summaries and errors/warnings of all files in a directory arranged in a single file](#get-summaries-and-errorswarnings-of-all-files-in-a-directory-arranged-in-a-single-file)
    - [Scenario](#scenario)
    - [Command](#command)
    - [Variations](#variations)
      - [I don't want warnings, only errors](#i-dont-want-warnings-only-errors)
      - [I want to view my.log in a text viewer but ANSI escape codes ruins the summary layout.](#i-want-to-view-mylog-in-a-text-viewer-but-ansi-escape-codes-ruins-the-summary-layout)
      - [No summary report in the file, just errors/warnings](#no-summary-report-in-the-file-just-errorswarnings)


# Basic





# Advanced

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

<summary>Show variations of this scenario</summary>

### Variations

#### I don't want warnings, only errors

Change `--verbosity 1` to `--verbosity 0`

#### I want to view my.log in a text viewer but ANSI escape codes ruins the summary layout.

Use an ANSI to HTML converter like [aha](https://github.com/theZiz/aha) (available through `apt` and `dnf`) or [ansi2html](https://pypi.org/project/ansi2html/) (`pip`).
Then alter the command to pipe to the converter e.g. with `ansi2html`:
```shell
find tests/test-data -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its-stave --verbosity 1 {} 2>&1 | ansi2html >> my.log.html ' {} \;
```
It can now be viewed in any browser.
> `aha --black` gives the same result as `ansi2html`.

#### No summary report in the file, just errors/warnings
Redirect `stderr` to my.log by removing `2>&1` and put `2` in front of the file appending `>>` i.e.
```shell
find tests/test-data -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its-stave --verbosity 1 {} 2>> my.log' {} \;
```
This will instead print the summaries to the terminal (stdout).

If you completely want to ignore the report summaries, different platforms have a way to mute stdout such as `/dev/null` on Unix-like. Below command is platform independent and just redirects stdout to `ignore.txt` (truncating).
```shell
find tests/test-data -type f -name "*.raw" -exec sh -c 'echo -e "\n\n--- {} ---\n" >> my.log; fastpasta check all its-stave --verbosity 1 {} 2>> my.log' {} > ignore.txt \;
```


</details>
