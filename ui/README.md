# Ui

Phoenix LiveView web interface with BPMN and Petri Net visualization that uses the Rust library.

## Development

### Phoenix

Install dependencies:

```bash
cd ui
sudo systemctl start postgresql.service
mix deps.get
mix phx.server
```

The application will be available at: `http://localhost:4000`.
