defmodule UiWeb.ConverterComponents do
  use Phoenix.Component

  attr(:bpmn_xml, :string, default: nil)
  attr(:pnml, :string, default: nil)
  attr(:dot, :string, default: nil)
  attr(:active_tab, :string, default: "graph")
  attr(:loading, :boolean, default: false)

  def converter(assigns) do
    ~H"""
    <div class="border rounded bg-gray-50 flex flex-col h-[70vh]">
      <%= cond do %>
        <% @loading -> %>
          <div class="flex-1 flex items-center justify-center text-gray-400">
            Converting...
          </div>

        <% is_nil(@dot) -> %>
          <div class="flex-1 flex items-center justify-center text-gray-400 text-lg">
            The Petri Net encoding will appear here
          </div>

        <% true -> %>
          <div class="flex items-center justify-between px-2 pt-2">
            <div class="flex space-x-2">
              <.tab_button label="PN Graph" tab="graph" active_tab={@active_tab} />
              <.tab_button label="DOT" tab="dot" active_tab={@active_tab} />
              <.tab_button label="PNML" tab="pnml" active_tab={@active_tab} />
              <.tab_button label="BPMN" tab="bpmn" active_tab={@active_tab} />
            </div>

            <div class="flex space-x-2 pb-1">
              <.download_link :if={@dot} content={@dot} filename="output.dot" label="Download DOT" />
              <.download_link :if={@pnml} content={@pnml} filename="output.pnml" label="Download PNML" />
            </div>
          </div>

          <div class="flex-1 min-h-0 border-t bg-white">
            <div class={["w-full h-full", @active_tab != "graph" && "hidden"]}>
              <.dot_graph_hook />
            </div>

            <.source_pane :if={@active_tab == "dot"} id="dot" content={@dot} />
            <.source_pane :if={@active_tab == "pnml"} id="pnml" content={@pnml} />
            <.source_pane :if={@active_tab == "bpmn"} id="bpmn" content={@bpmn_xml} />
          </div>
      <% end %>
    </div>
    """
  end

  # Kept minimal on purpose: a colocated hook script tag has to sit next to
  # a plain element, without surrounding cond/case blocks, or the macro
  # component parser fails.
  defp dot_graph_hook(assigns) do
    ~H"""
    <div id="dot-graph" phx-hook=".DotGraph" phx-update="ignore" class="relative w-full h-full">
    </div>

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
        // el is hidden (display:none ancestor) — skip, do not corrupt panZoom
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
          this.el.innerHTML = "<p class='text-red-600 p-3'>Cannot render DOT graph</p>";
          console.error(err);
          // Per il caveat noto di viz.js: dopo un errore l'istanza Viz
          // può restare in uno stato inutilizzabile, ne creiamo una nuova.
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
      phx-click="switch_tab"
      phx-value-tab={@tab}
      class={[
        "px-3 py-1 rounded-t font-semibold border border-b-0",
        @active_tab == @tab && "bg-white text-blue-700 border-blue-300",
        @active_tab != @tab && "bg-blue-100 text-gray-700 border-blue-200"
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
    href = "data:text/plain;charset=utf-8;base64," <> Base.encode64(assigns.content)
    assigns = assign(assigns, :href, href)

    ~H"""
    <a href={@href} download={@filename} class="btn btn-xs btn-outline">
      {@label}
    </a>
    """
  end

  attr(:id, :string, required: true)
  attr(:content, :string, required: true)

  defp source_pane(assigns) do
    ~H"""
    <pre id={"src-" <> @id} class="h-full w-full overflow-auto p-3 text-sm font-mono whitespace-pre-wrap"><code>{@content}</code></pre>
    """
  end
end
