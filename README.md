# kicad-parser

Parser for KiCad files

## Running

## Dev Commands 
Note: You may need to install `mise` (but you can also run alt cargo commands)
- Run tests - `mise test` (`cargo test`)
- Run fmt - `mise fmt-all` (`cargo +nightly fmt --all --`)

## Running the CLI

A simple `Clap`-based CLI is provided to run modifications

### Add a text Box!

This will add a text element to the schematic 

```
cargo run -p kicad-parser-cli -- -i test.kicad_sch -o test-2.kicad_sch text 'stuff and things' --align top right
```

### Remove a property 

This will remove a property from a symbol

```
cargo run -p kicad-parser-cli -- -i test.kicad_sch -o test-2.kicad_sch remove --symbol 'power:+3V3' --property '#PWR0301'
```

### Add a property 

This will add a property to a symbol 

```
cargo run -p kicad-parser-cli -- -i test.kicad_sch -o test-2.kicad_sch add --symbol 'power:+3V3' --type 'Thingy' --name 'Stuff' 
```
