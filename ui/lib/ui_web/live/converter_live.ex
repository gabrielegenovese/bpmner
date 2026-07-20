defmodule UiWeb.ConverterLive do
  use UiWeb, :live_view
  import UiWeb.ConverterComponents, only: [converter: 1, theme_toggle: 1]

  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:bpmn_xml, nil)
     |> assign(:pnml, nil)
     |> assign(:dot, nil)
     |> assign(:active_tab, "graph")
     |> assign(:loading, false)
     |> assign(:toast, nil)
     |> allow_upload(:bpmn, accept: :any, max_entries: 1)}
  end

  def render(assigns) do
    ~H"""
    <div class="max-w-5xl mx-auto p-6 space-y-6">
      <div class="toast toast-top toast-end">
        <div :if={@toast} class="alert alert-error shadow-lg">
          <span>{@toast}</span>
        </div>
      </div>
      <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">BPMN Chor <span class="text-primary">⇢</span> Petri Net</h1>
        <div class="flex gap-2 items-center">
          <.theme_toggle />
          <.link navigate={~p"/info"} class="btn btn-ghost btn-sm">Info</.link>
          <a href="https://github.com/" target="_blank" class="btn btn-ghost btn-sm">GitHub</a>
        </div>
      </div>

      <form
        id="bpmn-form"
        phx-submit="convert"
        phx-change="validate"
        class="flex flex-wrap gap-3 items-center"
      >
        <.live_file_input upload={@uploads.bpmn} class="file-input file-input-bordered" />
        <button type="submit" class="btn btn-primary" phx-disable-with="Encoding...">
          Encode
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
        case Converter.convert_bpmn_to_pnml(file) do
          {:error, reason} ->
            Process.send_after(self(), :clear_toast, 10000)

            {:noreply,
             socket
             |> assign(:toast, reason)}

          pnml ->
            case Converter.convert_bpmn_to_dot(file) do
              {:error, reason} ->
                Process.send_after(self(), :clear_toast, 10000)

                {:noreply,
                 socket
                 |> assign(:toast, reason)}

              dot ->
                {:noreply,
                 socket
                 |> assign(:bpmn_xml, file)
                 |> assign(:pnml, pnml)
                 |> assign(:dot, dot)
                 |> assign(:active_tab, "graph")
                 |> push_event("render_dot", %{dot: dot})
                 |> push_event("render_bpmn", %{xml: file})}
            end
        end

      [] ->
        {:noreply, socket}
    end
  end

  def handle_info(:clear_toast, socket) do
    {:noreply, assign(socket, :toast, nil)}
  end

  def handle_event("validate", _params, socket) do
    {:noreply, socket}
  end

  def handle_event("switch_tab", %{"tab" => tab}, socket) do
    socket = assign(socket, :active_tab, tab)

    socket =
      case tab do
        "graph" -> push_event(socket, "resize_dot", %{})
        "bpmn_diagram" -> push_event(socket, "resize_bpmn", %{})
        _ -> socket
      end

    {:noreply, socket}
  end
end
