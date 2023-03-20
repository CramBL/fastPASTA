# Checks performed
## Prelimary sanity checks (Performed in the `input module`)
1. `Once` The first 10 bytes of the input is read as an RDH0 and the version field is checked, if it is not 6 or 7, processing is stopped.

2. `Every RDH` The input scanner uses RDHs to navigate the data, and does one sanity check on the `offset_to_next` field. It subtracts the size of an RDH (64 bytes) from the value of the `offset_to_next` field, and checks that the result is not less than 0, and not more than 20 KB. If it fails, processing will stop.

# Running checks (Performed in the `validation module`)
## RDH running checks
### Check stop_bit and page_counter
The first 2 RDHs sets the baseline for the expected page_counter increments (if second rdh has page_counter == 2, then the increments are +2).

Uses the value of the stop_bit to determine if the page_counter is expected to increment or reset to 0.

* If stop_bit == 0
  * Check page_counter == expected_page_counter
  * Increment expected_page_counter
* If stop_bit == 1
  * Check page_counter == expected_page_counter
  * Reset expected_page_counter to 0


## Payload running checks
Before each payload is checked, the rdh for that payload is set as the current rdh. A state machine (see below) is used to keep track of which words are expected, and sanity checks are performed on the each word (sanity checks are listed further down this document).

Additional checks not performed in the sanity checks:
* Word is DDW0
  * RDH stop_bit == 1
  * RDH pages_counter $>$ 0


Certain transitions are ambigious (marked by yellow notes), these are resolved based on the ID of the next received GBT word.
![CDP FSM for validation](CDP_payload_StateMachine%20(continuous%20mode).png)


# Sanity checks (Performed in the `validation module`)
If any of the following conditions are not met, the RDH fails the sanity check and one or more error messages is printed to stderr.
## RDH sanity checks
* RDH0
  * Header ID equal to first Header ID seen during processing
  * header_size = 0x40
  * FeeID
    * 0 $\le$ layer $\le$ 6
    * 0 $\le$ stave $\le$ 47
    * reserved = 0
  * system_id = 0x20 `ITS system ID`
  * priority_bit = 0
  * reserved = 0
* RDH1
  * bc $\lt$ 0xdeb
  * reserved = 0
* RDH2
  * stop_bit $\le$ 1
  * trigger_type $\ge$ 1 AND all spare bits = 0
  * reserved = 0
* RDH3
  * reserved = 0 `includes reserved 23:4 in detector field`
* dw $\le$ 1
* data_format $\le$ 2

## Payload sanity checks
### Status Words
#### IHW
* id = 0xE0
* reserved = 0

#### TDH
* id = 0xE8
* reserved = 0
* trigger_type != 0 `OR` internal_trigger != 0

#### TDT
* id = 0xF0
* reserved = 0

#### DDW0
* id = 0xE4
* reserved = 0
* index $\ge$ 1

### Data Words
Checks that the ID is a valid ID for IL, ML or OL.

If any of the following checks passes, it is considered valid.

0x20 $\le$ ID $\le$ 0x28 `IL`

0x43 $\le$ ID $\le$ 0x46 `ML`

0x48 $\le$ ID $\le$ 0x4B `ML`

0x53 $\le$ ID $\le$ 0x56 `ML`

0x58 $\le$ ID $\le$ 0x5B `ML`

0x40 $\le$ ID $\le$ 0x46 `OL`

0x48 $\le$ ID $\le$ 0x4E `OL`

0x50 $\le$ ID $\le$ 0x56 `OL`

0x58 $\le$ ID $\le$ 0x5E `OL`