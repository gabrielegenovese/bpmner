defmodule UiWeb.ConverterLive do
  use UiWeb, :live_view
  import UiWeb.ConverterComponents, only: [converter: 1]

  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:bpmn_xml, nil)
     |> assign(:pnml, nil)
     |> assign(:dot, nil)
     |> assign(:active_tab, "graph")
     |> assign(:loading, false)
     |> allow_upload(:bpmn, accept: :any, max_entries: 1)}
  end

  def render(assigns) do
    ~H"""
    <div class="max-w-5xl mx-auto p-6 space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">BPMN Chor <span class="text-primary">⇢</span> Petri Net Converter</h1>
        <div class="flex gap-2">
          <.link navigate={~p"/info"} class="btn btn-ghost btn-sm">Info</.link>
          <a href="https://github.com/gabrielegenovese/bpmner" target="_blank" class="btn btn-ghost btn-sm">GitHub</a>
        </div>
      </div>

      <form
        id="bpmn-form"
        phx-submit="convert"
        phx-change="validate"
        class="flex flex-wrap gap-3 items-center"
      >
        <.live_file_input upload={@uploads.bpmn} class="file-input file-input-bordered" />
        <button type="submit" class="btn btn-primary" phx-disable-with="Converto...">
          Convert
        </button>
        <span :for={err <- upload_errors(@uploads.bpmn)} class="badge badge-error">
          {Phoenix.Naming.humanize(err)}
        </span>
      </form>

      <.converter
        bpmn_xml={@bpmn_xml}
        pnml={@pnml}
        dot={@dot}
        active_tab={@active_tab}
        loading={@loading}
      />
    </div>
    """
  end

  def handle_event("convert", _params, socket) do
    files =
      consume_uploaded_entries(socket, :bpmn, fn %{path: path}, _entry ->
        {:ok, File.read!(path)}
      end)

    case files do
      [file] ->
        pnml = Converter.convert_bpmn_to_pnml(file)
        dot = Converter.convert_bpmn_to_dot(file)

        {:noreply,
         socket
         |> assign(:bpmn_xml, file)
         |> assign(:pnml, pnml)
         |> assign(:dot, dot)
         |> assign(:active_tab, "graph")
         |> push_event("render_dot", %{dot: dot})}

      [] ->
        {:noreply, socket}
    end
  end

  def handle_event("validate", _params, socket) do
    {:noreply, socket}
  end

  def handle_event("switch_tab", %{"tab" => tab}, socket) do
    socket = assign(socket, :active_tab, tab)

    socket =
      if tab == "graph" do
        push_event(socket, "resize_dot", %{})
      else
        socket
      end

    {:noreply, socket}
  end
end
