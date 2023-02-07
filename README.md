# fastPASTA
## fast Protocol Analysis Scanner Tool for ALICE

For extensive documentation, invoke ```cargo doc --open```

`fastPASTA` follows the CLI guidelines for design https://clig.dev/.

## Purpose

To parse CRU Data Packets for protocol violations and report any errors

## To start using fastPASTA, simply execute fastpasta from the compiled binary in ../target/debug/fastpasta
### See help

```
./target/debug/fastpasta -h
```


### Print the first 10 bytes in 2 files:
```
./target/debug/fastpasta -b 10 ../Downloads/data_ols_ul.raw ../Downloads/data_ols_no_ul.raw
```
## Roadmap
Coming soon...

## License
MIT?

## Project status
Under development
