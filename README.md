# BPMN Choreography -> Petri Net Encoder

A web application and Rust library to encode **BPMN Choreography** diagrams into **Petri Nets**.

## Features

- Import BPMN Choreography (`.bpmn`)
- Encode BPMN to Petri Nets
- Export as:
  - PNML (`.pnml`)
  - Graphviz DOT (`.dot`)
- Rust conversion engine exposed to Elixir through Rustler NIFs
- Phoenix LiveView web interface with BPMN and Petri Net visualization

---

## Project Structure

```
.
├── crates/
│   ├── cli/          # Command-line interface
│   └── core/         # BPMN parser, encoder and Petri net library
├── examples/         # Example BPMN models
├── ui/
│   ├── assets/       # CSS and JavaScript
│   ├── config/       # Phoenix configuration
│   ├── lib/          # LiveView application
│   ├── native/       # Rustler NIF
│   └── priv/         # Static assets and compiled NIF
├── Cargo.toml        # Rust workspace
├── Dockerfile
├── docker-compose.yml
└── README.md
```

---

## Docker

Build the image:

```bash
docker compose build
```

Start the application:

```bash
docker compose up -d
```

Stop:

```bash
docker compose down
```