defmodule UiWeb.InfoLive do
  use UiWeb, :live_view

  def mount(_params, _session, socket) do
    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <div class="max-w-3xl mx-auto p-6 space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">About the translation</h1>
        <.link navigate={~p"/"} class="btn btn-ghost btn-sm">Back</.link>
      </div>

      <div class="prose max-w-none">
        <p>
          This tool translates BPMN choreography diagrams into Petri nets,
          exported as PNML and DOT graph representations.
        </p>

        <h2>Supported BPMN elements</h2>
        <ul>
          <li>Start / end events</li>
          <li>Tasks</li>
          <li>Exclusive gateways</li>
          <li>Parallel gateways</li>
          <!-- TODO: adjust this list to match wf_core's actual coverage -->
        </ul>

        <h2>Translation semantics</h2>
        <p>
          <!-- TODO: describe the encoding rules from wf_core::encoder::enc_chor -->
        </p>

        <h2>Known limitations</h2>
        <ul>
          <li>
            <!-- TODO -->
          </li>
        </ul>
      </div>
    </div>
    """
  end
end
