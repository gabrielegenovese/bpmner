defmodule UiWeb.ConverterComponents do
  use Phoenix.Component

  attr(:bpmn_xml, :string, default: nil)
  attr(:pnml, :string, default: nil)
  attr(:dot, :string, default: nil)
  attr(:active_tab, :string, default: "graph")
  attr(:loading, :boolean, default: false)

  attr(:class, :string, default: nil)

  def theme_toggle(assigns) do
    ~H"""
    <button
      type="button"
      class={["btn btn-ghost btn-xs btn-circle", @class]}
      title="Toggle theme"
      onclick="window.__setTheme(document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark')"
    >
      <svg class="h-5 w-5 fill-none stroke-current block dark:hidden" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path
          d="M12 3V4M12 20V21M4 12H3M6.31412 6.31412L5.5 5.5M17.6859 6.31412L18.5 5.5M6.31412 17.69L5.5 18.5001M17.6859 17.69L18.5 18.5001M21 12H20M16 12C16 14.2091 14.2091 16 12 16C9.79086 16 8 14.2091 8 12C8 9.79086 9.79086 8 12 8C14.2091 8 16 9.79086 16 12Z"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>

      <svg class="h-4 w-4 fill-current hidden dark:block" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <path d="M21.64 13a1 1 0 00-1.05-.14 8.05 8.05 0 01-3.37.73 8.15 8.15 0 01-8.14-8.1 8.59 8.59 0 01.25-2A1 1 0 008 2.36a10.14 10.14 0 1014 11.69 1 1 0 00-.36-1.05z" />
      </svg>
    </button>
    """
  end

  def converter(assigns) do
    ~H"""
    <div class="border border-base-300 rounded bg-base-200 flex flex-col h-[70vh]">
      <%= cond do %>
        <% @loading -> %>
          <div class="flex-1 flex items-center justify-center text-base-content/50">
            Converting...
          </div>

        <% is_nil(@dot) -> %>
          <div class="flex-1 flex items-center justify-center text-base-content/50 text-lg">
            The Petri Net encoding will appear here
          </div>

        <% true -> %>
          <div class="flex items-center justify-between px-2 pt-2">
            <div class="flex space-x-2">
              <.tab_button label="PN Graph" tab="graph" active_tab={@active_tab} />
              <.tab_button label="BPMN Diagram" tab="bpmn_diagram" active_tab={@active_tab} />
            </div>

            <div class="flex space-x-2 pb-1">
              <.download_link :if={@dot} content={@dot} filename="output.dot" label="Download DOT" />
              <.download_link :if={@pnml} content={@pnml} filename="output.pnml" label="Download PNML" />
            </div>
          </div>

          <div class="flex-1 min-h-0 border-t border-base-300 bg-base-100">
            <div class={["w-full h-full bg-white", @active_tab != "graph" && "hidden"]}>
              <.dot_graph_hook />
            </div>

            <div class={["w-full h-full bg-white", @active_tab != "bpmn_diagram" && "hidden"]}>
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
    <div id="bpmn-viewer" phx-hook=".BpmnViewer" phx-update="ignore" class="relative w-full h-full"></div>

    <style>
      #bpmn-viewer { touch-action:none; user-select:none; cursor:grab; }
    </style>

    <script :type={Phoenix.LiveView.ColocatedHook} name=".BpmnViewer">
      export default {
        mounted() {
          this.viewer = new ChorJS({ container: this.el });

          this.handleEvent("render_bpmn", ({ xml }) => this.renderBpmn(xml));
          this.handleEvent("resize_bpmn", () => requestAnimationFrame(() => this.fit()));

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
      phx-click="switch_tab"
      phx-value-tab={@tab}
      class={[
        "px-3 py-1 rounded-t font-semibold border border-b-0",
        @active_tab == @tab && "bg-base-100 text-primary border-base-300",
        @active_tab != @tab && "bg-base-200 text-base-content/70 border-base-300"
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
      href={@href}
      download={@filename}
      class="btn btn-xs btn-outline"
      :if={is_binary(@content)}
    >
      {@label}
    </a>
    """
  end

  attr(:id, :string, required: true)
  attr(:content, :string, required: true)

  defp source_pane(assigns) do
    ~H"""
    <pre id={"src-" <> @id} class="h-full w-full overflow-auto p-3 text-sm font-mono whitespace-pre-wrap bg-base-100 text-base-content"><code>{@content}</code></pre>
    """
  end
end
