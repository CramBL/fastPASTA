# [ALICE](https://home.cern/science/experiments/alice) Protocol Reader
[![coverage report](https://gitlab.cern.ch/mkonig/fastpasta/badges/master/coverage.svg)](https://gitlab.cern.ch/mkonig/fastpasta/-/commits/master)
## Purpose
Provide a simple and efficient reader (stdio/file), that let's a user read the raw binary protocol of the [ALICE Detector](https://home.cern/science/experiments/alice)'s readout system into a convenient data structure for further analysis.

- [ALICE Protocol Reader](#alice-protocol-reader)
  - [Purpose](#purpose)
- [Example](#example)
  - [Customize InputScanner behaviour with a config](#customize-inputscanner-behaviour-with-a-config)


# Example
First add the `alice_protocol_reader` crate to your project
```shell
$ cargo add alice_protocol_reader
```
Then use the convenience `init_reader()`-function to add the appropriate reader (stdin or file) at runtime. Instantiate the `InputScanner` with the reader and start reading ALICE data.
```rust
use input_scanner::InputScanner;
use alice_protocol_reader::init_reader;
use rdh::RdhCru;

let reader = init_reader(&Some(test_file_path)).unwrap();

let mut input_scanner = InputScanner::minimal(reader);

let rdh = input_scanner.load_rdh_cru::<RdhCru<u8>>().unwrap();

println!("{rdh:?}");
```
Example output

```
RdhCru
        Rdh0 { header_id: 7, header_size: 64, fee_id: 20522, priority_bit: 0, system_id: 32, reserved0: 0 }
        offset_new_packet: 5088
        memory_size: 5088
        link_id: 0
        packet_counter: 0
        cruid_dw: 24
        Rdh1 { bc_reserved0: 0, orbit: 192796021 }
        dataformat_reserved0: 2
        Rdh2 { trigger_type: 27139, pages_counter: 0, stop_bit: 0, reserved0: 0 }
        reserved1: 0
        Rdh3 { detector_field: 0, par_bit: 0, reserved0: 0 }
        reserved2: 0
```

## Customize InputScanner behaviour with a config

Implement the `FilterOpt` on your own config struct and pass it to the `InputScanner` to customize its behaviour

```Rust
use alice_protocol_reader::filter::FilterOpt;

struct MyCfg;

impl FilterOpt for MyCfg {
    fn skip_payload(&self) -> bool {
        // Implement your config rules for determining if you're skipping the payload (only reading `RDH`s)
    }

    fn filter_link(&self) -> Option<u8> {
        // Implement your config rules for setting a link to filter by
    }

    fn filter_fee(&self) -> Option<u16> {
        // Implement your config rules for setting a FEE ID to filter by
    }

    fn filter_its_stave(&self) -> Option<u16> {
        // Implement your config rules for setting an ITS Stave to filter by
    }
}

use input_scanner::InputScanner;
use alice_protocol_reader::init_reader;
use rdh::RdhCru;
pub fn main() {
    let reader = init_reader(&Some(test_file_path)).unwrap();

    let mut input_scanner = input_scanner::InputScanner::new(&MyCfg, reader, None); // None: Option<flume::Sender<InputStatType>>

    let rdh = input_scanner.load_cdp::<RdhCru<u8>>();
}
```
