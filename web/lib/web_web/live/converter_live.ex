defmodule WebWeb.ConverterLive do
  use WebWeb, :live_view

  def mount(_params, _session, socket) do
    {:ok,
    socket
    |> assign(:result, nil)
    |> allow_upload(:bpmn,
      accept: :any,
      max_entries: 1
    )}
  end

  def render(assigns) do
    ~H"""
    <h1> BPMN Converter </h1>

    <form id="bpmn-form"
          phx-submit="convert"
          phx-change="validate">

      <.live_file_input upload={@uploads.bpmn}/>

      <button type="submit">
        Converti
      </button>

    </form>

    <%= if @result do %>
      <pre><%= @result %></pre>
    <% end %>
    """
  end


  def handle_event("convert", _params, socket) do
    files =
      consume_uploaded_entries(socket, :bpmn, fn %{path: path}, _entry ->
        {:ok, File.read!(path)}
      end)

    case files do
      [file] ->
        result = Converter.convert_bpmn_to_pnml(file)
        {:noreply, assign(socket, :result, result)}

      [] ->
        {:noreply,
        assign(socket, :result, "Nessun file caricato")}
    end
  end

  def handle_event("validate", _params, socket) do
    {:noreply, socket}
  end
end
