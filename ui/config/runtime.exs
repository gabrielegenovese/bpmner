import Config

# Start the endpoint when explicitly requested (`mix phx.server`, releases).
if System.get_env("PHX_SERVER") do
  config :ui, UiWeb.Endpoint, server: true
end

# TINA's `sift` binary used for the optional safety / deadlock analysis.
# The Docker image installs it at /usr/local/bin/sift; set SIFT_PATH to use a
# different binary.
config :ui, :sift_path, System.get_env("SIFT_PATH") || "sift"

# Safety/deadlock analysis is bounded so that a pathological model cannot block
# the UI forever. Raise these limits with the environment variables below.
config :ui, :analysis_max_states, String.to_integer(System.get_env("SIFT_MAX_STATES") || "200000")

config :ui, :analysis_timeout, String.to_integer(System.get_env("SIFT_TIMEOUT") || "15")

if config_env() == :prod do
  secret_key_base = System.fetch_env!("SECRET_KEY_BASE")
  host = System.get_env("PHX_HOST") || "localhost"
  http_port = String.to_integer(System.get_env("PORT") || "4000")
  url_scheme = System.get_env("PHX_SCHEME") || "http"
  url_port = String.to_integer(System.get_env("PHX_URL_PORT") || Integer.to_string(http_port))

  config :ui, :dns_cluster_query, System.get_env("DNS_CLUSTER_QUERY")

  config :ui, UiWeb.Endpoint,
    url: [host: host, port: url_port, scheme: url_scheme],
    http: [ip: {0, 0, 0, 0}, port: http_port],
    secret_key_base: secret_key_base,
    server: true
end

if config_env() == :dev do
  # Reload browser tabs when matching files change.
  config :ui, UiWeb.Endpoint,
    live_reload: [
      web_console_logger: true,
      patterns: [
        # Static assets, except user uploads
        ~r"priv/static/(?!uploads/).*\.(js|css|png|jpeg|jpg|gif|svg)$"E,
        # Router, Controllers, LiveViews and LiveComponents
        ~r"lib/ui_web/router\.ex$"E,
        ~r"lib/ui_web/(controllers|live|components)/.*\.(ex|heex)$"E
      ]
    ]
end
