FROM elixir:1.20.2-slim

RUN apt-get update && \
    apt-get install -y build-essential git nodejs npm curl openssl && \
    rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:$PATH"
ENV MIX_ENV=prod
ENV PHX_SERVER=true

WORKDIR /app

COPY . .

WORKDIR /app/ui

RUN mix local.hex --force
RUN mix local.rebar --force
RUN mix deps.get --only prod
RUN mix compile
RUN mix assets.setup
RUN mix assets.deploy
RUN mix release

EXPOSE 4000

CMD ["_build/prod/rel/ui/bin/ui", "start"]