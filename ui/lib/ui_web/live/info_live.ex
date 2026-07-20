defmodule UiWeb.InfoLive do
  use UiWeb, :live_view
  import UiWeb.ConverterComponents, only: [converter: 1, theme_toggle: 1]

  def mount(_params, _session, socket) do
    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <div class="max-w-3xl mx-auto p-6 space-y-8">
      <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">Encoding Details</h1>
        <div class="flex gap-2 items-center">
          <.theme_toggle />
          <.link navigate={~p"/"} class="btn btn-ghost btn-sm">Encoder</.link>
          <a href="https://github.com/" target="_blank" class="btn btn-ghost btn-sm">GitHub</a>
        </div>
      </div>

      <section class="space-y-2">
        <p class="text-base-content/80">
          This tool translates BPMN choreography diagrams into Petri nets,
          based on the encoding described in the <a href="https://hal.science/hal-05665560" target="_blank" class="link link-primary">paper</a>
          <em>"A Petri Net Semantics for BPMN Choreographies"</em>
          (Genovese, Di Giusto, Lanese, Tuosto).
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Supported BPMN elements</h2>
        <ul class="list-disc list-inside space-y-1 text-base-content/80">
          <li>Start and end events</li>
          <li>Tasks (sender / receiver interactions)</li>
          <li>AND-split / AND-join (parallel gateway)</li>
          <li>XOR-split / XOR-join (exclusive gateway)</li>
          <li>OR-split / OR-join (inclusive gateway)</li>
        </ul>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">How the encoding works</h2>
        <p class="text-base-content/80">
          Each BPMN element is mapped to a Petri net fragment (places, transitions,
          and flow arcs). Start/end events, tasks, and AND-gateways are encoded directly.
          XOR- and OR-gateways additionally rely on a technique called
          <strong>Dead Path Elimination (DPE)</strong>: whenever a split gateway can choose
          not to activate a branch, a parallel "dead propagation net" carries tokens along
          the discarded paths.
        </p>
        <p class="text-base-content/80">
          The corresponding OR-join can then determine locally, from the tokens it
          receives, whether every incoming branch has been resolved as either live or
          dead — without needing any global knowledge of the process state.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">OR-gateway encoding</h2>
        <p class="text-base-content/80">
          For an OR-gateway with <em>n</em> outgoing (or incoming) branches, the encoding
          introduces one transition per non-empty subset of branches, representing every
          possible combination of activated paths. This means the number of transitions
          grows exponentially in the number of branches — in practice not an issue, since
          BPMN models rarely have more than a handful of branches per gateway.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Guarantees</h2>
        <p class="text-base-content/80">
          The encoding is proven <strong>correct</strong> and <strong>complete</strong> with
          respect to the reference operational semantics for BPMN choreographies
          (Corradini et al.): every step in the BPMN semantics corresponds to a firing in
          the Petri net, and vice versa.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Known limitations</h2>
        <ul class="list-disc list-inside space-y-1 text-base-content/80">
          <li>Only acyclic choreographies are supported (no loops)</li>
          <li>
            Models must be <strong>well-formed</strong>: every edge is used exactly once,
            there is a single start event and at least one end event, and every element
            must be reachable
          </li>
          <li>
            Models must be <strong>safe</strong>: no gateway can be activated twice
            concurrently
          </li>
          <li>Mixed gateways (combined split/join in one node) are not supported</li>
        </ul>
      </section>
    </div>
    """
  end
end
