# The full (non-slim) Elixir image already ships the toolchain we need:
# gcc/make, git, curl, openssl, libssl-dev and the CA certificates. That avoids
# running apt at build time, which is unreliable on some hosts.
FROM elixir:1.20.2

# TINA release used for the safety / deadlock-freedom checks. The binaries are
# published for amd64 and arm64, so the same Dockerfile works on both.
ARG TINA_VERSION=4.0.0
ARG TARGETARCH

# Install TINA's `sift` state-space explorer so that the web interface can
# analyse the encoded Petri nets out of the box.
RUN arch="${TARGETARCH:-$(uname -m)}"; \
    case "$arch" in \
        amd64|x86_64) tina_arch=amd64 ;; \
        arm64|aarch64) tina_arch=arm64 ;; \
        *) echo "Unsupported architecture for TINA: $arch" >&2; exit 1 ;; \
    esac; \
    curl -fsSL \
        "https://projects.laas.fr/tina/binaries/tina-${TINA_VERSION}-${tina_arch}-linux.tgz" \
        -o /tmp/tina.tgz; \
    tar -xzf /tmp/tina.tgz -C /tmp; \
    install -m 0755 "/tmp/tina-${TINA_VERSION}/bin/sift" /usr/local/bin/sift; \
    rm -rf /tmp/tina.tgz "/tmp/tina-${TINA_VERSION}"

ENV SIFT_PATH=/usr/local/bin/sift

# esbuild is normally downloaded from the npm registry by `mix assets.setup`.
# Fetch the pinned standalone binary from a CDN instead, so the build does not
# depend on the registry being reachable.
ARG ESBUILD_VERSION=0.25.4

RUN arch="${TARGETARCH:-$(uname -m)}"; \
    case "$arch" in \
        amd64|x86_64) esbuild_pkg=linux-x64 ;; \
        arm64|aarch64) esbuild_pkg=linux-arm64 ;; \
        *) echo "Unsupported architecture for esbuild: $arch" >&2; exit 1 ;; \
    esac; \
    curl -fsSL \
        "https://cdn.jsdelivr.net/npm/@esbuild/${esbuild_pkg}@${ESBUILD_VERSION}/bin/esbuild" \
        -o /usr/local/bin/esbuild; \
    chmod 0755 /usr/local/bin/esbuild; \
    /usr/local/bin/esbuild --version

ENV MIX_ESBUILD_PATH=/usr/local/bin/esbuild

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --profile minimal

ENV PATH="/root/.cargo/bin:$PATH"
ENV MIX_ENV=prod
ENV PHX_SERVER=true
ENV PORT=4000

# `force_ssl` is compile-time in Phoenix. Keep it disabled by default so the
# container can be reached directly over HTTP; enable it when running behind a
# TLS-terminating reverse proxy with `--build-arg FORCE_SSL=true`.
ARG FORCE_SSL=false
ENV FORCE_SSL=${FORCE_SSL}

WORKDIR /app

COPY . .

WORKDIR /app/ui

RUN mix local.hex --force && \
    mix local.rebar --force && \
    mix deps.get --only prod && \
    mix compile && \
    mix assets.setup && \
    mix assets.deploy && \
    mix release

# Run as an unprivileged user. The release needs write access to its own
# directory (for the Erlang cookie/tmp files) and to /tmp, where the analysis
# writes the temporary PNML documents.
RUN useradd --create-home --shell /bin/bash app && \
    chown -R app:app /app

USER app

EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -fsS "http://127.0.0.1:${PORT}/" >/dev/null || exit 1

CMD ["_build/prod/rel/ui/bin/ui", "start"]
