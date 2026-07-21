# Ui

Phoenix LiveView web interface with BPMN and Petri Net visualization that uses the Rust library.

## Development

### Phoenix

Install dependencies:

```bash
sudo systemctl start postgresql.service
mix deps.get
mix phx.server
```

The application will be available at: `http://localhost:4000`.

## Todos

- button to redirect dot using `rankdir="LR"`
- ready examples to select
- better info
- better logging of tool
- model and live encoding? (advanced)
- analysis using other tools (advanced)