# UI

Phoenix LiveView web interface for BPMN choreography to Petri net encoding,
with BPMN and Petri net visualization.

## Requirements

- Elixir 1.17+ and Erlang/OTP
- A Rust toolchain (the Rustler NIF is compiled by `mix compile`)
- Optional: [TINA](https://projects.laas.fr/tina/) for the safety and
  deadlock-freedom checks (already included in the Docker image)

## Development

Install dependencies:

```bash
mix deps.get
```

Start the application:

```bash
SIFT_PATH=/path/to/sift mix phx.server
```

The application will be available at <http://localhost:4000>.

`SIFT_PATH` is only needed when `sift` is not on `PATH`. The analysis is
optional: without TINA the encoder and exports still work, and the analysis
panel shows an explanatory warning.

## Features

- Upload a `.bpmn` choreography or load one of the bundled examples.
- View the BPMN diagram and the generated Petri net graph.
- Download PNML and DOT exports.
- Toggle the initial marking included in exports.
- Toggle the DOT layout between top-to-bottom and left-to-right.
- Automatically check the encoded net with TINA's `sift`:
  - safety (`L.safe`);
  - deadlock-freedom, accepting the intended final marking.

The network request does not block the LiveView: analysis runs in a separate
task and its result is sent back to the process.

## Configuration

All settings are read from the environment at runtime:

| Variable | Default | Meaning |
| --- | --- | --- |
| `SIFT_PATH` | `sift` | Path or name of the TINA `sift` binary |
| `SIFT_MAX_STATES` | `200000` | Maximum number of markings to explore |
| `SIFT_TIMEOUT` | `15` | Exploration time limit, in seconds |
| `PORT` | `4000` | HTTP port |

## Docker

The image defined at the repository root builds this application together with
the Rust NIF and TINA 4.0.0, and runs it as an unprivileged user. From the
repository root:

```bash
cp .env.example .env   # then edit SECRET_KEY_BASE and PHX_HOST
docker compose up -d --build
```

See the root `README.md` for the complete deployment and HTTPS configuration.

## Bundled examples

The `priv/examples` directory contains:

- the pizzeria choreography with AND gateways;
- the paper example with XOR/OR gateways.

## Formatting

```bash
mix format
```
