# Rust libraries

Rust library to encode BPMN to Petri Net. The folder `core` contains the main 
logic of the encoding. The folder `cli` contains and Command-line interface
to easily use the library.

## Development

### Run

```bash
cargo run convert ../../examples/chor/example-chor.bpmn
```

## Todos

- detect collab and first give error then extend when theory is ready 
- improve chor parsing ans errors managing
- extend examples
- interaction with petri net tools