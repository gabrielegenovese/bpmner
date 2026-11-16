defmodule UiWeb.ConverterLive do
  use UiWeb, :live_view

  import UiWeb.ConverterComponents,
    only: [converter: 1, theme_toggle: 1, analysis_panel: 1, github_link: 1]

  require Logger

  @default_max_states 200_000
  @default_timeout 15

  @impl true
  # The initial marking is always part of the exports, so it is not exposed as
  # an option in the UI.
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
    <div class="mx-auto max-w-6xl space-y-5 px-5 py-8 sm:px-6">
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

      <header class="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 class="text-3xl font-semibold tracking-tight">bpmner</h1>
          <p class="mt-0.5 text-sm text-base-content/60">
            BPMN Choreography <span class="font-medium text-primary">⇢</span> Petri Net
          </p>
        </div>
        <nav class="flex items-center gap-1">
          <.theme_toggle />
          <.link navigate={~p"/info"} class="btn btn-ghost btn-sm">Info</.link>
          <.github_link />
        </nav>
      </header>

      <section class="rounded-2xl border border-base-300/80 bg-gradient-to-r from-primary/5 via-base-100 to-secondary/10 p-3 shadow-sm">
        <div class="flex flex-col gap-3 lg:flex-row lg:items-center">
          <form phx-change="select_example" class="w-full lg:w-60">
            <select
              name="example"
              class="select select-sm w-full border-base-300 bg-base-200/40 focus:border-primary"
              aria-label="Load an example"
            >
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

          <div class="hidden h-6 w-px shrink-0 bg-base-300 lg:block" aria-hidden="true"></div>

          <form
            id="bpmn-form"
            phx-submit="convert"
            phx-change="validate"
            class="flex min-w-0 flex-1 flex-col gap-2 sm:flex-row sm:items-center"
          >
            <.live_file_input
              upload={@uploads.bpmn}
              class="file-input file-input-sm w-full text-xs sm:w-56"
            />
            <div class="flex items-center gap-2">
              <button
                type="submit"
                class="btn btn-primary btn-sm shadow-md shadow-primary/20"
                phx-disable-with="Encoding..."
              >
                Encode
              </button>
              <span :for={err <- upload_errors(@uploads.bpmn)} class="badge badge-error badge-sm">
                {Phoenix.Naming.humanize(err)}
              </span>
            </div>
          </form>

          <div class="flex items-center gap-3 border-t border-base-300/70 pt-3 lg:ml-auto lg:border-0 lg:pt-0">
            <label
              class="flex cursor-pointer items-center gap-2 text-xs text-base-content/70"
              title="Run TINA/sift automatically after encoding"
            >
              <input
                type="checkbox"
                class="toggle toggle-primary toggle-xs"
                checked={@auto_analyze}
                phx-click="toggle_auto_analyze"
              /> Auto-analyze
            </label>
          </div>
        </div>

        <p :if={@selected_example} class="mt-2 px-1 text-xs text-base-content/50">
          {selected_description(@examples, @selected_example)}
        </p>
      </section>

      <.converter
        bpmn_xml={@bpmn_xml}
        pnml={@pnml}
        dot={@dot}
        active_tab={@active_tab}
        loading={@loading}
        rankdir_lr={@rankdir_lr}
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

  def handle_event("toggle_auto_analyze", _params, socket) do
    auto_analyze = !socket.assigns.auto_analyze
    socket = assign(socket, :auto_analyze, auto_analyze)

    if auto_analyze and socket.assigns.bpmn_xml do
      {:noreply, start_analysis(socket)}
    else
      {:noreply, socket}
    end
  end

  def handle_event("set_layout", %{"layout" => layout}, socket) do
    rankdir = layout == "horizontal"

    if rankdir == socket.assigns.rankdir_lr do
      {:noreply, socket}
    else
      socket = assign(socket, :rankdir_lr, rankdir)

      case socket.assigns.bpmn_xml do
        nil ->
          {:noreply, socket}

        xml ->
          case Converter.convert_bpmn_to_dot(xml, true, rankdir) do
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

    with {:ok, pnml} <- Converter.convert_bpmn_to_pnml(xml, true),
         {:ok, dot} <-
           Converter.convert_bpmn_to_dot(xml, true, socket.assigns.rankdir_lr) do
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
