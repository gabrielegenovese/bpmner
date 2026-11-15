defmodule UiWeb.ConverterLive do
  use UiWeb, :live_view
  import UiWeb.ConverterComponents, only: [converter: 1, theme_toggle: 1, analysis_panel: 1]

  require Logger

  @default_max_states 200_000
  @default_timeout 15

  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:bpmn_xml, nil)
     |> assign(:pnml, nil)
     |> assign(:dot, nil)
     |> assign(:active_tab, "graph")
     |> assign(:loading, false)
     |> assign(:toast, nil)
     |> assign(:examples, Ui.Examples.all())
     |> assign(:selected_example, nil)
     |> assign(:initial_marking, true)
     |> assign(:rankdir_lr, false)
     |> assign(:auto_analyze, true)
     |> assign(:analysis, nil)
     |> assign(:analysis_loading, false)
     |> assign(:analysis_error, nil)
     |> assign(:sift_path, Application.get_env(:ui, :sift_path, "sift"))
     |> assign(
       :analysis_max_states,
       Application.get_env(:ui, :analysis_max_states, @default_max_states)
     )
     |> assign(:analysis_timeout, Application.get_env(:ui, :analysis_timeout, @default_timeout))
     |> allow_upload(:bpmn, accept: :any, max_entries: 1)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="max-w-5xl mx-auto p-6 space-y-6">
      <div class="toast toast-bottom toast-end z-50">
        <div
          :if={@toast}
          class="alert alert-error shadow-xl w-80 rounded-2xl items-start gap-3 p-4"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6 shrink-0 stroke-current mt-0.5"
            fill="none"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"
            />
          </svg>

          <div class="flex-1">
            <p class="font-semibold text-sm">Error</p>
            <p class="text-sm opacity-90">{@toast}</p>
          </div>

          <button
            type="button"
            phx-click="dismiss_toast"
            class="btn btn-ghost btn-xs btn-circle shrink-0"
            aria-label="Dismiss"
          >
            ✕
          </button>
        </div>
      </div>
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold">bpmner</h1>
          <p class="text-base-content/70">
            BPMN Choreography <span class="text-primary">⇢</span> Petri Net
          </p>
        </div>
        <div class="flex gap-2 items-center">
          <.theme_toggle />
          <.link navigate={~p"/info"} class="btn btn-ghost btn-sm">Info</.link>
          <a
            href="https://github.com/gabrielegenovese/bpmner"
            target="_blank"
            class="btn btn-ghost btn-sm"
          >GitHub</a>
        </div>
      </div>

      <div class="flex flex-wrap gap-3 items-center">
        <form phx-change="select_example">
          <select name="example" class="select select-sm select-bordered" aria-label="Load an example">
            <option value="">Load an example…</option>
            <option
              :for={example <- @examples}
              value={example.id}
              selected={@selected_example == example.id}
            >
              {example.name}
            </option>
          </select>
        </form>

        <form
          id="bpmn-form"
          phx-submit="convert"
          phx-change="validate"
          class="flex flex-wrap gap-3 items-center"
        >
          <.live_file_input
            upload={@uploads.bpmn}
            class="file-input file-input-sm file-input-bordered"
          />
          <button type="submit" class="btn btn-primary btn-sm" phx-disable-with="Encoding...">
            Encode
          </button>
          <span :for={err <- upload_errors(@uploads.bpmn)} class="badge badge-error">
            {Phoenix.Naming.humanize(err)}
          </span>
        </form>

        <button
          type="button"
          phx-click="analyze"
          class="btn btn-outline btn-sm"
          disabled={is_nil(@bpmn_xml) or @analysis_loading}
          title="Run TINA/sift on the encoded Petri net"
        >
          {if @analysis_loading, do: "Analyzing...", else: "Analyze"}
        </button>

        <label class="label cursor-pointer gap-2 py-0">
          <input
            type="checkbox"
            class="checkbox checkbox-xs"
            checked={@auto_analyze}
            phx-click="toggle_auto_analyze"
          />
          <span class="label-text text-xs">Analyze automatically</span>
        </label>
      </div>

      <div :if={@selected_example} class="text-xs text-base-content/60">
        {selected_description(@examples, @selected_example)}
      </div>

      <.converter
        bpmn_xml={@bpmn_xml}
        pnml={@pnml}
        dot={@dot}
        active_tab={@active_tab}
        loading={@loading}
        rankdir_lr={@rankdir_lr}
        initial_marking={@initial_marking}
      />

      <.analysis_panel
        analysis={@analysis}
        loading={@analysis_loading}
        error={@analysis_error}
        sift_path={@sift_path}
      />
    </div>
    """
  end

  @impl true
  def handle_event("convert", _params, socket) do
    files =
      consume_uploaded_entries(socket, :bpmn, fn %{path: path}, _entry ->
        {:ok, File.read!(path)}
      end)

    case files do
      [file] ->
        {:noreply, convert(socket, file)}

      [] ->
        {:noreply, put_toast(socket, "Please choose a BPMN file first.")}
    end
  end

  @impl true
  def handle_event("select_example", %{"example" => ""}, socket) do
    {:noreply, assign(socket, :selected_example, nil)}
  end

  def handle_event("select_example", %{"example" => id}, socket) do
    case Ui.Examples.get(id) do
      nil ->
        {:noreply, put_toast(socket, "Unknown example: #{id}")}

      example ->
        case Ui.Examples.read(example) do
          {:ok, xml} ->
            {:noreply,
             socket
             |> assign(:selected_example, id)
             |> convert(xml, example.name)}

          {:error, message} ->
            {:noreply, put_toast(socket, message)}
        end
    end
  end

  def handle_event("analyze", _params, socket) do
    {:noreply, start_analysis(socket)}
  end

  def handle_event("toggle_auto_analyze", _params, socket) do
    {:noreply, assign(socket, :auto_analyze, !socket.assigns.auto_analyze)}
  end

  def handle_event("toggle_rankdir", _params, socket) do
    rankdir = !socket.assigns.rankdir_lr
    socket = assign(socket, :rankdir_lr, rankdir)

    case socket.assigns.bpmn_xml do
      nil ->
        {:noreply, socket}

      xml ->
        case Converter.convert_bpmn_to_dot(xml, socket.assigns.initial_marking, rankdir) do
          {:ok, dot} ->
            {:noreply,
             socket
             |> assign(:dot, dot)
             |> push_event("render_dot", %{dot: dot})}

          {:error, reason} ->
            {:noreply, put_toast(socket, reason)}
        end
    end
  end

  def handle_event("toggle_initial_marking", _params, socket) do
    marking = !socket.assigns.initial_marking
    socket = assign(socket, :initial_marking, marking)

    case socket.assigns.bpmn_xml do
      nil -> {:noreply, socket}
      xml -> {:noreply, reencode(socket, xml)}
    end
  end

  def handle_event("dismiss_toast", _params, socket) do
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

  @impl true
  def handle_info({:analysis_done, result, duration}, socket) do
    socket = assign(socket, :analysis_loading, false)

    case result do
      {:ok, report} ->
        Logger.info(
          "petri-net analysis finished in #{duration}ms " <>
            "(safety=#{get_in(report, ["safety", "status"])}, " <>
            "deadlock=#{get_in(report, ["deadlock", "status"])}, " <>
            "states=#{get_in(report, ["safety", "states"])}, " <>
            "places=#{report["places"]}, transitions=#{report["transitions"]})"
        )

        {:noreply, assign(socket, :analysis, report)}

      {:error, reason} ->
        Logger.warning("petri-net analysis unavailable: #{reason}")
        {:noreply, assign(socket, :analysis_error, reason)}
    end
  end

  def handle_info(:clear_toast, socket) do
    {:noreply, assign(socket, :toast, nil)}
  end

  defp convert(socket, xml, source \\ "upload") do
    started = System.monotonic_time(:millisecond)

    socket =
      assign(socket,
        loading: true,
        toast: nil,
        analysis: nil,
        analysis_error: nil
      )

    with {:ok, pnml} <-
           Converter.convert_bpmn_to_pnml(xml, socket.assigns.initial_marking),
         {:ok, dot} <-
           Converter.convert_bpmn_to_dot(
             xml,
             socket.assigns.initial_marking,
             socket.assigns.rankdir_lr
           ) do
      duration = System.monotonic_time(:millisecond) - started

      Logger.info(
        "encoded #{source} in #{duration}ms " <>
          "(bpmn=#{byte_size(xml)}B, pnml=#{byte_size(pnml)}B, dot=#{byte_size(dot)}B)"
      )

      socket
      |> assign(
        bpmn_xml: xml,
        pnml: pnml,
        dot: dot,
        loading: false,
        active_tab: "graph"
      )
      |> push_event("render_dot", %{dot: dot})
      |> push_event("render_bpmn", %{xml: xml})
      |> maybe_auto_analyze()
    else
      {:error, reason} ->
        Logger.warning("encoding failed for #{source}: #{reason}")

        socket
        |> assign(loading: false)
        |> put_toast(reason)
    end
  end

  # Re-export without resetting the analysis until the new export is ready.
  defp reencode(socket, xml) do
    started = System.monotonic_time(:millisecond)

    with {:ok, pnml} <-
           Converter.convert_bpmn_to_pnml(xml, socket.assigns.initial_marking),
         {:ok, dot} <-
           Converter.convert_bpmn_to_dot(
             xml,
             socket.assigns.initial_marking,
             socket.assigns.rankdir_lr
           ) do
      duration = System.monotonic_time(:millisecond) - started

      Logger.info("re-encoded net in #{duration}ms")

      socket
      |> assign(pnml: pnml, dot: dot)
      |> push_event("render_dot", %{dot: dot})
      |> maybe_auto_analyze()
    else
      {:error, reason} ->
        put_toast(socket, reason)
    end
  end

  defp maybe_auto_analyze(socket) do
    if socket.assigns.auto_analyze do
      start_analysis(socket)
    else
      socket
    end
  end

  defp start_analysis(socket) do
    case socket.assigns.bpmn_xml do
      nil ->
        socket

      xml ->
        parent = self()
        sift = socket.assigns.sift_path
        max_states = socket.assigns.analysis_max_states
        timeout = socket.assigns.analysis_timeout

        Task.start(fn ->
          started = System.monotonic_time(:millisecond)

          result =
            try do
              Ui.Analysis.run(xml, sift, max_states, timeout)
            rescue
              e -> {:error, Exception.message(e)}
            end

          send(parent, {:analysis_done, result, System.monotonic_time(:millisecond) - started})
        end)

        assign(socket,
          analysis_loading: true,
          analysis: nil,
          analysis_error: nil
        )
    end
  end

  defp put_toast(socket, message) do
    Process.send_after(self(), :clear_toast, 10_000)
    assign(socket, :toast, message)
  end

  defp selected_description(examples, id) do
    case Enum.find(examples, &(&1.id == id)) do
      nil -> nil
      example -> example.description
    end
  end
end
