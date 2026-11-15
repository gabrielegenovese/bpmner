# BPMN Choreography → Petri Net Encoder

A Rust library, command-line tool and Phoenix LiveView web application that
encode **BPMN Choreography** diagrams into **Petri Nets**.

The translation follows the encoding presented in
[*A Petri Net Semantics for Acyclic Safe BPMN Choreographies*](https://hal.science/hal-05665560),
and supports start/end events, choreography tasks, and AND, XOR and OR gateways.

## Features

- Parse BPMN choreography models and report precise, actionable errors.
- Encode choreographies into Petri nets with Dead Path Elimination.
- Include the **initial marking** in PNML and DOT exports by default.
- Export PNML and Graphviz DOT, with optional left-to-right DOT layout.
- Detect BPMN collaborations and reject them with a dedicated error instead of
  silently producing an incorrect net.
- Check the encoded net with [TINA](https://projects.laas.fr/tina/)'s `sift`
  state-space explorer: **safety** (`L.safe`) and **deadlock-freedom**, allowing
  the intended final marking.
- Phoenix web interface with BPMN and Petri net visualization, bundled examples,
  live encoding options and automatic analysis.
- Self-contained Docker image (amd64/arm64) with TINA already installed.

## Project structure

```
.
├── crates/
│   ├── cli/          # Command-line interface and HTTP server
│   └── core/         # BPMN parser, encoder, Petri net library and analysis
├── examples/         # Example BPMN models
├── ui/
│   ├── assets/       # CSS and JavaScript
│   ├── config/       # Phoenix configuration
│   ├── lib/          # LiveView application
│   ├── native/       # Rustler NIF
│   └── priv/         # Static assets, examples and compiled NIF
├── Cargo.toml        # Rust workspace
├── Dockerfile
├── docker-compose.yml
├── .env.example      # Environment template for Docker deployments
└── README.md
```

## Command line

Convert a model:

```bash
cargo run -p cli -- convert examples/chor/pizzeria.bpmn \
  --pnml output/pizzeria.pnml \
  --dot output/pizzeria.dot
```

The initial marking is included by default. Use `--no-initial-marking` to export
an unmarked skeleton, and `--rankdir-lr` to lay the DOT graph out from left to
right.

Analyze the encoded net with TINA:

```bash
cargo run -p cli -- check examples/chor/pizzeria.bpmn \
  --sift /path/to/sift \
  --max-states 200000 \
  --timeout 15
```

The `check` command exits with status 1 when a property is violated, which makes
it suitable for CI. The `--sift` path can also be supplied through the
`SIFT_PATH` environment variable. If TINA is installed and `sift` is on `PATH`,
the option can be omitted.

The CLI also provides an HTTP server:

```bash
cargo run -p cli -- serve --port 8080
```

It exposes `POST /convert` and `POST /analyze` with a multipart `file` field.
`/convert` accepts `format=pnml|dot|both`, `rankdir=lr` and
`initial_marking=false`; `/analyze` accepts `sift`, `max_states` and `timeout`.

## Web interface

```bash
cd ui
mix deps.get
SIFT_PATH=/path/to/sift mix phx.server
```

Open <http://localhost:4000>. The interface provides:

- direct upload of a `.bpmn` file or selection of a bundled example;
- BPMN and Petri net graph views, with a downloadable PNML and DOT export;
- a switch between vertical and horizontal DOT layouts;
- automatic safety/deadlock analysis, with the explored state count, a witness
  state for violations, and the raw analyzer log.

The analysis is optional: if `sift` is unavailable, the rest of the tool keeps
working and the analysis panel explains how to install TINA or set `SIFT_PATH`.
The exploration can be bounded with `SIFT_MAX_STATES` (default `200000`) and
`SIFT_TIMEOUT` (default `15` seconds).

## TINA analysis

`sift` reads the PNML document produced by the encoder. The formulas are:

- safety: `L.safe`, asserting that no place ever carries more than one token;
- deadlock-freedom: `-L.dead \/ (final)`, where `final` is the conjunction of
  `-S.<place>` over every place. This accepts the intended final marking, in
  which all places are empty, and reports any other deadlock.

Install TINA from <https://projects.laas.fr/tina/> or point `SIFT_PATH` at an
existing `sift` binary. The Docker image already includes TINA 4.0.0, so no
extra setup is required there.

## Docker deployment

The image is self-contained: it builds the Rust NIF and the Phoenix release and
installs TINA 4.0.0 (`sift`), so safety and deadlock analysis work without any
extra setup. Both `amd64` and `arm64` hosts are supported.

1. Create the environment file and edit it:

   ```bash
   cp .env.example .env
   ```

   At minimum, set `SECRET_KEY_BASE` (generate it with `openssl rand -base64 48`,
   or with `mix phx.gen.secret` inside `ui/`) and `PHX_HOST` (your domain or IP).

2. Build and start the container:

   ```bash
   docker compose up -d --build
   ```

3. Open `http://<PHX_HOST>:<HOST_PORT>` (default port `4000`).

Useful commands:

```bash
docker compose ps            # status and health check
docker compose logs -f ui    # application logs
docker compose down          # stop the application
git pull && docker compose up -d --build   # update
```

### Docker configuration

All settings are read from `.env` (see `.env.example`):

| Variable | Default | Meaning |
| --- | --- | --- |
| `SECRET_KEY_BASE` | — | Required application secret |
| `PHX_HOST` | `localhost` | Public host name or IP used in URLs |
| `BIND_ADDRESS` | `0.0.0.0` | Host address the published port binds to |
| `HOST_PORT` | `4000` | Host port mapped to the container |
| `PHX_SCHEME` | `http` | Scheme used in generated URLs |
| `PHX_URL_PORT` | `4000` | Port used in generated URLs |
| `FORCE_SSL` | `false` | Redirect HTTP to HTTPS; build-time option |
| `SIFT_PATH` | `/usr/local/bin/sift` | TINA `sift` binary bundled in the image (fixed in compose) |
| `SIFT_MAX_STATES` | `200000` | Maximum markings explored by the analysis |
| `SIFT_TIMEOUT` | `15` | Analysis time limit in seconds |

### HTTPS behind a reverse proxy

To terminate TLS in Caddy, Nginx or Traefik and forward to this container, set
in `.env`:

```dotenv
PHX_HOST=example.org
BIND_ADDRESS=127.0.0.1
PHX_SCHEME=https
PHX_URL_PORT=443
FORCE_SSL=true
```

Then rebuild and restart:

```bash
docker compose up -d --build
```

`FORCE_SSL` is applied at build time (Phoenix reads it at compile time), so a
rebuild is required after changing it.

## Tests

```bash
cargo test
cd ui && mix compile
```
