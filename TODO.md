# TODO

## UI

- [x] Add a full-screen button for the encoding output and the BPMN visualizer.
- [ ] Bundle chor-js locally instead of loading it from jsDelivr, so the BPMN
      diagram also works without internet access.
- [ ] Add UI tests covering example loading, encoding, downloads and analysis
      states.
- [ ] Show the analysis limits (max states, timeout) and let advanced users
      change them from the UI.
- [ ] Add copy-to-clipboard buttons for the PNML and DOT sources.

## Infrastructure

- [x] Add CI for `cargo fmt`, `cargo clippy`, `cargo test`, `mix format`,
      `mix compile`, `mix test` and the Docker build.
- [ ] Move the runtime stage to a slimmer base image: the current `elixir`
      image still ships build tools that the release does not need.
- [ ] Enable BuildKit cache mounts for Hex, Cargo and `_build` to speed up
      cold rebuilds further.
