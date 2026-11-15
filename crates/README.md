# Rust libraries

Rust library and command-line interface to encode BPMN choreographies into
Petri nets.

- `core` contains the BPMN parser, the encoder, the Petri net representation,
  the PNML/DOT exporters and the TINA analysis integration.
- `cli` exposes the library through a command-line tool and a small HTTP server.

## Development

### Convert

```bash
cargo run -p cli -- convert examples/chor/pizzeria.bpmn \
  --pnml output/pizzeria.pnml \
  --dot output/pizzeria.dot
```

The initial marking is included by default. Use `--no-initial-marking` to omit
it. Use `--rankdir-lr` to lay out the DOT graph from left to right.

If no output path is given, the encoded Petri net is printed to standard output.

### Check

```bash
cargo run -p cli -- check examples/chor/pizzeria.bpmn \
  --sift /path/to/sift \
  --max-states 200000 \
  --timeout 15
```

The command checks safety with the formula `L.safe` and deadlock-freedom with
`-L.dead \/ (final)`, where the intended final marking has all places empty.
The exit status is 1 when a property is violated.

`--sift` can be replaced by the `SIFT_PATH` environment variable, or omitted
when `sift` is on `PATH`. Install TINA from <https://projects.laas.fr/tina/>;
the Docker image at the repository root already includes it for the web UI.

### HTTP server

```bash
cargo run -p cli -- serve --port 8080
```

- `POST /convert` with a multipart `file` field; query parameters `format`
  (`pnml`, `dot`, `both`), `rankdir` (`lr`) and `initial_marking` (`false`).
- `POST /analyze` with a multipart `file` field; query parameters `sift`,
  `max_states` and `timeout`.

## Tests

```bash
cargo test -p core
```

## Supported input

Only BPMN **choreographies** are accepted. Collaborations, process-only models,
sub-choreographies, event-based gateways and intermediate events are detected
and reported with a dedicated error. Choreographies must be well-formed,
acyclic and structurally safe; malformed models are rejected before a Petri net
is produced.
