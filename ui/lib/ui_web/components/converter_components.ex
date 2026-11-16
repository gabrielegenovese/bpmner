defmodule UiWeb.ConverterComponents do
  use Phoenix.Component

  def brand_mark(assigns) do
    ~H"""
    <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-accent text-primary-content shadow-md shadow-primary/20">
      <svg class="h-5 w-5 fill-current" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <circle cx="6" cy="5" r="2.5" />
        <circle cx="6" cy="19" r="2.5" />
        <circle cx="18" cy="12" r="2.5" />
        <path
          d="M8.5 6.5 15 10.5M8.5 17.5 15 13.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
      </svg>
    </div>
    """
  end

  attr(:class, :string, default: nil)

  def theme_toggle(assigns) do
    ~H"""
    <button
      type="button"
      class={["btn btn-ghost btn-xs btn-circle", @class]}
      title="Toggle theme"
      onclick="window.__setTheme(document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark')"
    >
      <svg
        class="h-5 w-5 fill-none stroke-current block dark:hidden"
        viewBox="0 0 24 24"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M12 3V4M12 20V21M4 12H3M6.31412 6.31412L5.5 5.5M17.6859 6.31412L18.5 5.5M6.31412 17.69L5.5 18.5001M17.6859 17.69L18.5 18.5001M21 12H20M16 12C16 14.2091 14.2091 16 12 16C9.79086 16 8 14.2091 8 12C8 9.79086 9.79086 8 12 8C14.2091 8 16 9.79086 16 12Z"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>

      <svg
        class="h-4 w-4 fill-current hidden dark:block"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
      >
        <path d="M21.64 13a1 1 0 00-1.05-.14 8.05 8.05 0 01-3.37.73 8.15 8.15 0 01-8.14-8.1 8.59 8.59 0 01.25-2A1 1 0 008 2.36a10.14 10.14 0 1014 11.69 1 1 0 00-.36-1.05z" />
      </svg>
    </button>
    """
  end

  attr(:bpmn_xml, :string, default: nil)
  attr(:pnml, :string, default: nil)
  attr(:dot, :string, default: nil)
  attr(:active_tab, :string, default: "graph")
  attr(:loading, :boolean, default: false)
  attr(:rankdir_lr, :boolean, default: false)

  def converter(assigns) do
    ~H"""
    <div
      id="converter-panel"
      phx-hook="Fullscreen"
      data-fullscreen-root
      class="flex h-[70vh] flex-col overflow-hidden rounded-2xl border border-base-300/80 bg-base-100 shadow-sm ring-1 ring-primary/5"
    >
      <%= cond do %>
        <% @loading -> %>
          <div class="flex flex-1 items-center justify-center text-sm text-base-content/40">
            Converting…
          </div>
        <% is_nil(@dot) -> %>
          <div class="flex flex-1 items-center justify-center p-8 text-center text-sm text-base-content/40">
            Upload a BPMN choreography or load an example to see the Petri net.
          </div>
        <% true -> %>
          <div class="flex flex-wrap items-center justify-between gap-2 border-b border-base-300/70 bg-base-200/30 px-2 py-2">
            <div class="inline-flex rounded-lg bg-base-200 p-0.5" role="tablist">
              <.tab_button label="Petri net" tab="graph" active_tab={@active_tab} />
              <.tab_button label="BPMN" tab="bpmn_diagram" active_tab={@active_tab} />
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <div :if={@active_tab == "graph"} class="flex items-center gap-2">
                <div
                  class="inline-flex items-center rounded-lg bg-base-200 p-0.5"
                  role="group"
                  aria-label="Graph layout"
                >
                  <.layout_button label="Vertical" layout="vertical" active={!@rankdir_lr} />
                  <.layout_button label="Horizontal" layout="horizontal" active={@rankdir_lr} />
                </div>

                <span class="hidden h-5 w-px bg-base-300 sm:block" aria-hidden="true"></span>
              </div>

              <.download_link :if={@dot} content={@dot} filename="output.dot" label="DOT" />
              <.download_link :if={@pnml} content={@pnml} filename="output.pnml" label="PNML" />

              <button
                type="button"
                data-fullscreen-toggle
                aria-pressed="false"
                class="btn btn-ghost btn-xs btn-circle text-base-content/60 hover:text-primary"
                title="Enter full screen"
              >
                <svg
                  class="fullscreen-expand h-4 w-4 fill-none stroke-current"
                  viewBox="0 0 24 24"
                  stroke-width="1.8"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15" />
                </svg>
                <svg
                  class="fullscreen-compress h-4 w-4 fill-none stroke-current"
                  viewBox="0 0 24 24"
                  stroke-width="1.8"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M9 3.75v4.5m0-4.5H4.5m4.5 0L3.75 9M9 20.25v-4.5m0 4.5H4.5m4.5 0l-5.25-5.25M20.25 9h-4.5m4.5 0V4.5m0 4.5L15 3.75M20.25 15h-4.5m4.5 0v4.5m0-4.5L15 20.25" />
                </svg>
              </button>
            </div>
          </div>

          <div class="min-h-0 flex-1 bg-base-100">
            <div class={["h-full w-full bg-white", @active_tab != "graph" && "hidden"]}>
              <.dot_graph_hook />
            </div>

            <div class={["h-full w-full bg-white", @active_tab != "bpmn_diagram" && "hidden"]}>
              <.bpmn_viewer_hook />
            </div>

            <.source_pane :if={@active_tab == "bpmn"} id="bpmn" content={@bpmn_xml} />
          </div>
      <% end %>
    </div>
    """
  end

  defp bpmn_viewer_hook(assigns) do
    ~H"""
    <div id="bpmn-viewer" phx-hook=".BpmnViewer" phx-update="ignore" class="relative w-full h-full">
    </div>

    <style>
      #bpmn-viewer { touch-action:none; user-select:none; cursor:grab; }
    </style>

    <script :type={Phoenix.LiveView.ColocatedHook} name=".BpmnViewer">
      export default {
        mounted() {
          this.viewer = new ChorJS({ container: this.el });

          this.handleEvent("render_bpmn", ({ xml }) => this.renderBpmn(xml));
          this.handleEvent("resize_bpmn", () => requestAnimationFrame(() => this.fit()));

          this.onFullscreenChange = () => requestAnimationFrame(() => this.fit());
          document.addEventListener("fullscreenchange", this.onFullscreenChange);

          this.drag = false;
          this.last = null;

          this.el.addEventListener("pointerdown", e => {
            this.drag = true;
            this.last = [e.clientX, e.clientY];
            this.el.setPointerCapture(e.pointerId);
          });

          this.el.addEventListener("pointermove", e => {
            if (!this.drag) return;

            const canvas = this.viewer.get("canvas");
            const vb = canvas.viewbox();

            canvas.viewbox({
              x: vb.x - (e.clientX - this.last[0]) / vb.scale,
              y: vb.y - (e.clientY - this.last[1]) / vb.scale,
              width: vb.width,
              height: vb.height
            });

            this.last = [e.clientX, e.clientY];
          });

          this.el.addEventListener("pointerup", () => {
            this.drag = false;
          });

          this.el.addEventListener("wheel", e => {
            e.preventDefault();

            const canvas = this.viewer.get("canvas");
            canvas.zoom(canvas.zoom() + (e.deltaY > 0 ? -.1 : .1));
          }, { passive:false });

          window.addEventListener("keydown", e => {
            const canvas = this.viewer.get("canvas");

            if (e.key === "0") this.fit();
            if (e.key === "+") canvas.zoom(canvas.zoom() + .1);
            if (e.key === "-") canvas.zoom(canvas.zoom() - .1);
          });
        },

        destroyed() {
          document.removeEventListener("fullscreenchange", this.onFullscreenChange);
          this.viewer?.destroy();
        },

        fit() {
          const canvas = this.viewer.get("canvas");

          const rect = this.el.getBoundingClientRect();

          if (!rect.width || !rect.height) {
            requestAnimationFrame(() => this.fit());
            return;
          }

          try {
            canvas.zoom("fit-viewport");
          } catch (e) {
            console.error(e);
          }
        },

        async renderBpmn(xml) {
          if (!xml) return;

          try {
            await this.viewer.importXML(xml);

            requestAnimationFrame(() => {
              requestAnimationFrame(() => this.fit());
            });

          } catch(err) {
            this.el.innerHTML = "<p class='text-error p-3'>Cannot render BPMN diagram</p>";
            console.error(err);
          }
        }
      }
    </script>
    """
  end

  defp dot_graph_hook(assigns) do
    ~H"""
    <div id="dot-graph" phx-hook=".DotGraph" phx-update="ignore" class="relative w-full h-full"></div>

    <script :type={Phoenix.LiveView.ColocatedHook} name=".DotGraph">
        export default {
        mounted() {
          this.viz = new Viz();
          this.panZoom = null;

          this.resizeObserver = new ResizeObserver(() => {
            const svg = this.el.querySelector("svg");
            if (svg) this.sizeSvg(svg);
          });
          this.resizeObserver.observe(this.el);

          this.handleEvent("render_dot", ({ dot }) => this.renderDot(dot));

          this.handleEvent("resize_dot", () => {
            requestAnimationFrame(() => {
              const svg = this.el.querySelector("svg");
              if (svg) this.sizeSvg(svg);
            });
          });
        },

        destroyed() {
          if (this.resizeObserver) this.resizeObserver.disconnect();
          this.safeDestroyPanZoom();
        },

        safeDestroyPanZoom() {
          if (this.panZoom) {
            try {
              this.panZoom.destroy();
            } catch (_e) {
              // panZoom might already be in a broken/detached state, ignore
            }
            this.panZoom = null;
          }
        },

        forceSize(svg) {
          const { width, height } = this.el.getBoundingClientRect();
          if (width === 0 || height === 0) return;

          svg.setAttribute("width", width);
          svg.setAttribute("height", height);
          svg.style.width = width + "px";
          svg.style.height = height + "px";
        },

        sizeSvg(svg) {
          const { width, height } = this.el.getBoundingClientRect();
          if (width === 0 || height === 0) return;

          svg.style.position = "absolute";
          svg.style.top = "0";
          svg.style.left = "0";
          svg.setAttribute("width", width);
          svg.setAttribute("height", height);
          svg.style.width = width + "px";
          svg.style.height = height + "px";

          if (this.panZoom) {
            this.panZoom.resize();
            this.panZoom.fit();
            this.panZoom.center();
          }
        },

        async renderDot(dot) {
          if (!dot) return;

          try {
            const svgMarkup = await this.viz.renderString(dot);
            this.el.innerHTML = svgMarkup;

            const svg = this.el.querySelector("svg");
            svg.style.position = "absolute";
            svg.style.top = "0";
            svg.style.left = "0";

            this.forceSize(svg);
            await new Promise(requestAnimationFrame);

            this.safeDestroyPanZoom();

            this.panZoom = svgPanZoom(svg, {
              zoomEnabled: true,
              controlIconsEnabled: true,
              minZoom: 0.1,
              fit: true,
              center: true
            });

            this.forceSize(svg);
          } catch (err) {
            this.el.innerHTML = "<p class='text-error p-3'>Cannot render DOT graph</p>";
            console.error(err);
            this.viz = new Viz();
          }
        }
      }
    </script>
    """
  end

  attr(:label, :string, required: true)
  attr(:tab, :string, required: true)
  attr(:active_tab, :string, required: true)

  defp tab_button(assigns) do
    ~H"""
    <button
      type="button"
      role="tab"
      phx-click="switch_tab"
      phx-value-tab={@tab}
      aria-selected={to_string(@active_tab == @tab)}
      class={[
        "rounded-md px-3 py-1 text-xs font-medium transition-colors",
        @active_tab == @tab && "bg-primary/10 text-primary shadow-sm",
        @active_tab != @tab && "text-base-content/60 hover:bg-base-100/60 hover:text-base-content"
      ]}
    >
      {@label}
    </button>
    """
  end

  attr(:label, :string, required: true)
  attr(:layout, :string, required: true)
  attr(:active, :boolean, required: true)

  defp layout_button(assigns) do
    ~H"""
    <button
      type="button"
      phx-click="set_layout"
      phx-value-layout={@layout}
      aria-pressed={to_string(@active)}
      class={[
        "rounded-md px-2.5 py-1 text-xs font-medium transition-colors",
        @active && "bg-primary/10 text-primary shadow-sm",
        !@active && "text-base-content/60 hover:bg-base-100/60 hover:text-base-content"
      ]}
    >
      {@label}
    </button>
    """
  end

  attr(:content, :string, required: true)
  attr(:filename, :string, required: true)
  attr(:label, :string, required: true)

  defp download_link(assigns) do
    href =
      if is_binary(assigns.content) do
        "data:text/plain;charset=utf-8;base64," <> Base.encode64(assigns.content)
      else
        "#"
      end

    assigns = assign(assigns, :href, href)

    ~H"""
    <a
      :if={is_binary(@content)}
      href={@href}
      download={@filename}
      class="btn btn-ghost btn-xs gap-1.5 text-base-content/55 hover:bg-primary/5 hover:text-primary"
    >
      <svg
        class="h-3.5 w-3.5 fill-current shrink-0"
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 122.88 88.36"
      >
        <path d="M58.29,42.08V3.12C58.29,1.4,59.7,0,61.44,0s3.15,1.4,3.15,3.12v38.96L79.1,29.4c1.3-1.14,3.28-1.02,4.43,0.27 s1.03,3.25-0.27,4.39L63.52,51.3c-1.21,1.06-3.01,1.03-4.18-0.02L39.62,34.06c-1.3-1.14-1.42-3.1-0.27-4.39 c1.15-1.28,3.13-1.4,4.43-0.27L58.29,42.08L58.29,42.08L58.29,42.08z M0.09,47.43c-0.43-1.77,0.66-3.55,2.43-3.98 c1.77-0.43,3.55,0.66,3.98,2.43c1.03,4.26,1.76,7.93,2.43,11.3c3.17,15.99,4.87,24.57,27.15,24.57h52.55 c20.82,0,22.51-9.07,25.32-24.09c0.67-3.6,1.4-7.5,2.44-11.78c0.43-1.77,2.21-2.86,3.98-2.43c1.77,0.43,2.85,2.21,2.43,3.98 c-0.98,4.02-1.7,7.88-2.36,11.45c-3.44,18.38-5.51,29.48-31.8,29.48H36.07C8.37,88.36,6.3,77.92,2.44,58.45 C1.71,54.77,0.98,51.08,0.09,47.43L0.09,47.43z" />
      </svg>
      {@label}
    </a>
    """
  end

  attr(:id, :string, required: true)
  attr(:content, :string, required: true)

  defp source_pane(assigns) do
    ~H"""
    <pre
      id={"src-" <> @id}
      class="h-full w-full overflow-auto p-3 text-sm font-mono whitespace-pre-wrap bg-base-100 text-base-content"
    ><code>{@content}</code></pre>
    """
  end

  attr(:analysis, :map, default: nil)
  attr(:loading, :boolean, default: false)
  attr(:error, :string, default: nil)
  attr(:sift_path, :string, default: "sift")

  def analysis_panel(assigns) do
    ~H"""
    <div class="space-y-3 rounded-2xl border border-base-300/80 bg-base-100 p-4 shadow-sm">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <h2 class="flex items-center gap-2 text-sm font-semibold">
          <span class="inline-block h-2 w-2 rounded-full bg-gradient-to-br from-primary to-accent"></span>
          Petri net analysis
        </h2>
        <span class="text-xs text-base-content/50">TINA / sift</span>
      </div>

      <%= cond do %>
        <% @loading -> %>
          <p class="text-sm text-base-content/60 flex items-center gap-2">
            <span class="loading loading-spinner loading-xs"></span> Exploring the state space…
          </p>
        <% @error -> %>
          <div class="alert alert-warning items-start gap-3 py-2">
            <div class="flex-1 text-sm">
              <p class="font-semibold">Analyzer unavailable</p>
              <p class="opacity-90">{@error}</p>
              <p class="mt-1 text-xs opacity-75">
                Install TINA (projects.laas.fr/tina) and make sure the <code class="mx-1">sift</code>
                binary is on <code>PATH</code>, or set <code>SIFT_PATH</code>
                (currently: <code>{@sift_path}</code>).
              </p>
            </div>
          </div>
        <% is_nil(@analysis) -> %>
          <p class="text-sm text-base-content/60">
            Safety and deadlock-freedom are checked automatically after encoding.
          </p>
        <% @analysis["tool_error"] -> %>
          <div class="alert alert-warning items-start gap-3 py-2">
            <div class="flex-1 text-sm">
              <p class="font-semibold">Analyzer unavailable</p>
              <p class="opacity-90">{@analysis["error"]}</p>
            </div>
          </div>
        <% true -> %>
          <div class="grid gap-3 sm:grid-cols-2">
            <.property_card result={@analysis["safety"]} />
            <.property_card result={@analysis["deadlock"]} />
          </div>

          <p class="text-xs text-base-content/60">
            {@analysis["places"]} places, {@analysis["transitions"]} transitions.
            Safety means no place ever carries more than one token; deadlock-freedom
            allows the intended final marking, where all places are empty.
          </p>

          <details class="text-xs">
            <summary class="cursor-pointer text-base-content/60">Analyzer log</summary>
            <pre class="mt-2 max-h-56 overflow-auto rounded bg-base-100 p-2 text-[11px] whitespace-pre-wrap">{analysis_log(@analysis)}</pre>
          </details>
      <% end %>
    </div>
    """
  end

  attr(:result, :map, required: true)

  defp property_card(assigns) do
    ~H"""
    <div class={[
      "space-y-1 rounded-xl border p-3 transition-colors",
      property_class(@result["status"])
    ]}>
      <div class="flex items-center justify-between gap-2">
        <p class="text-sm font-medium">{@result["property"]}</p>
        <span class={["badge badge-sm", status_class(@result["status"])]}>
          {@result["status"]}
        </span>
      </div>

      <p :if={@result["violation"]} class="text-xs text-error">
        violating state: <code>{@result["violation"]}</code>
      </p>

      <p :if={@result["states"]} class="text-xs text-base-content/60">
        {@result["states"]} markings explored
      </p>
    </div>
    """
  end

  defp status_class("holds"), do: "badge-success"
  defp status_class("violated"), do: "badge-error"
  defp status_class(_), do: "badge-warning"

  defp property_class("holds"), do: "border-success/30 bg-success/5"
  defp property_class("violated"), do: "border-error/30 bg-error/5"
  defp property_class(_), do: "border-warning/30 bg-warning/5"

  defp analysis_log(analysis) do
    [analysis["safety"], analysis["deadlock"]]
    |> Enum.map(&(&1 && &1["log"]))
    |> Enum.reject(&is_nil/1)
    |> Enum.join("\n\n")
  end
end
