defmodule UiWeb.InfoLive do
  use UiWeb, :live_view
  import UiWeb.ConverterComponents, only: [theme_toggle: 1]

  def mount(_params, _session, socket) do
    {:ok, socket}
  end

  def render(assigns) do
    ~H"""
    <div class="mx-auto max-w-3xl space-y-8 px-5 py-8 sm:px-6">
      <header class="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 class="text-3xl font-semibold tracking-tight">bpmner</h1>
          <p class="mt-1 text-sm text-base-content/60">Encoding Details</p>
        </div>
        <nav class="flex items-center gap-1">
          <.theme_toggle />
          <.link navigate={~p"/"} class="btn btn-ghost btn-sm">Encoder</.link>
          <a
            href="https://github.com/gabrielegenovese/bpmner"
            target="_blank"
            class="btn btn-ghost btn-sm"
          >GitHub</a>
        </nav>
      </header>

      <section class="space-y-2">
        <p class="text-base-content/80">
          This tool encodes BPMN choreography diagrams into Petri nets. The
          translation follows the encoding introduced in the paper
          "A Petri Net Semantics for BPMN Choreographies"<.ref n="1" />,
          which gives a formal, provably correct semantics to BPMN choreographies
          by mapping them onto standard Petri nets.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Supported BPMN Choreography elements</h2>
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
          Every BPMN element is translated into a small Petri net fragment: a set
          of places, transitions, and flow arcs. Start and end events, tasks, and
          AND-gateways are encoded directly, with a one-to-one correspondence
          between the BPMN element and its Petri net counterpart.
        </p>
        <p class="text-base-content/80">
          XOR- and OR-gateways are more delicate, since a split gateway can choose
          not to activate some of its outgoing branches. To handle this, the
          encoding relies on a technique known as
          <strong>Dead Path Elimination (DPE)</strong><.ref n="2" />: alongside
          the "live" part of the net, a parallel <em>dead propagation net</em>
          carries
          tokens along the branches that were not selected. When a branch is
          skipped, a token still flows through its dead propagation counterpart,
          marking that path as resolved even though no live token ever reaches it.
        </p>
        <p class="text-base-content/80">
          Thanks to this mechanism, an OR-join can determine locally, just by
          looking at the tokens it receives, whether every incoming branch has
          been resolved (either as live or as dead). This avoids any need for
          global knowledge of the process state, which is what makes the OR-join
          gateway notoriously hard to formalize in the BPMN standard.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">OR-gateway encoding</h2>
        <p class="text-base-content/80">
          For an OR-gateway with <em>n</em> outgoing (or incoming) branches, the
          encoding introduces one transition for every non-empty subset of
          branches, one for each possible combination of activated paths. The
          number of transitions therefore grows exponentially with the number of
          branches.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Initial marking and exports</h2>
        <p class="text-base-content/80">
          The start event produces a dedicated place carrying one token in the <strong>initial marking</strong>. The marking is always included in
          both the PNML and DOT exports, so the net is immediately usable by
          external Petri net tools.
        </p>
        <p class="text-base-content/80">
          The DOT graph can be laid out top-to-bottom or <strong>left-to-right</strong>
          with the layout switch in the toolbar. Both exports can be downloaded
          from the same toolbar.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Automatic analysis with TINA</h2>
        <p class="text-base-content/80">
          After encoding, the net is checked with <a
            href="https://projects.laas.fr/tina/"
            target="_blank"
            class="link link-primary"
          >
            TINA
          </a>'s <code>sift</code>
          state-space explorer:
        </p>
        <ul class="list-disc list-inside space-y-1 text-base-content/80">
          <li>
            <strong>Safety</strong>: no place ever carries more than one token
            (<code>L.safe</code>);
          </li>
          <li>
            <strong>Deadlock-freedom</strong>: every deadlock is the intended final
            marking, in which all places are empty.
          </li>
        </ul>
        <p class="text-base-content/80">
          Results show the number of explored markings and, when a property is
          violated, a witness state. The raw analyzer log is available in the
          collapsible panel, and each run is also written to the application log.
          The analysis needs the <code>sift</code>
          binary on <code>PATH</code>: set <code>SIFT_PATH</code>
          to point at it otherwise. You can bound the
          exploration with <code>SIFT_MAX_STATES</code>
          and <code>SIFT_TIMEOUT</code>.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Bundled examples</h2>
        <p class="text-base-content/80">
          The example selector loads a few ready-made models: the pizzeria
          choreography with AND gateways and the paper example with XOR/OR
          gateways.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Guarantees</h2>
        <p class="text-base-content/80">
          The encoding is proven <strong>correct</strong>
          and <strong>complete</strong>, considering the limitation presented later,
          with respect to a reference operational
          semantics for BPMN choreographies<.ref n="3" />: every step allowed by
          the BPMN semantics corresponds to a firing in the encoded Petri net, and
          every firing in the net corresponds to a valid BPMN step.
        </p>
      </section>

      <section class="space-y-2">
        <h2 class="text-xl font-semibold border-b pb-1">Known limitations</h2>
        <p class="text-base-content/80">
          These are the current limitations of the tool:
          <ul class="list-disc list-inside space-y-1">
            <li>
              Models must be <strong>well-formed</strong>: every edge is used exactly
              once, there is a single start event and at least one end event, and
              every element is reachable from the start event
            </li>
            <li>
              Only <strong>choreographies</strong> are accepted: BPMN collaborations,
              processes, sub-choreographies and event-based gateways are rejected
              with a dedicated error message
            </li>
            <li>
              <strong>Mixed gateways</strong>, combining split and join behaviour in a single node, are not supported
            </li>
            <li>
              Models must be <strong>safe</strong>: no gateway can be activated twice
              concurrently; the integrated analysis reports a violation when this
              happens, and a <code>not_</code> place exceeding one token is the
              usual symptom
            </li>
            <li>
              Only <strong>acyclic</strong> choreographies are supported, loops are not handled yet
            </li>
          </ul>
          Structural problems (well-formedness, loops, unsupported elements,
          collaborations) are reported in the encoder before any Petri net is
          produced.
        </p>
      </section>

      <section id="references" class="space-y-2 pt-4">
        <h2 class="text-xl font-semibold border-b pb-1">References</h2>
        <ol class="list-decimal list-inside space-y-1 text-sm text-base-content/70">
          <li id="ref-1">
            G. Genovese, C. Di Giusto, I. Lanese, E. Tuosto.
            <em>A Petri Net Semantics for BPMN Choreographies.</em>
            <a href="https://hal.science/hal-05665560" target="_blank" class="link link-primary">
              hal.science/hal-05665560
            </a>
          </li>
          <li id="ref-2">
            M. Weidlich, A. Grosskopf, A. Barros. <em>Realising dead path elimination in BPMN.</em>
            IEEE Conference on Commerce and Enterprise Computing, 2009.
          </li>
          <li id="ref-3">
            F. Corradini, C. Muzi, B. Re, L. Rossi, F. Tiezzi.
            <em>BPMN 2.0 OR-Join Semantics: Global and local characterisation.</em>
            Information Systems 105, 2022.
          </li>
        </ol>
      </section>
    </div>
    """
  end

  attr(:n, :string, required: true)

  defp ref(assigns) do
    ~H"""
    <a href={"#ref-#{@n}"} class="align-super text-xs text-primary no-underline">[{@n}]</a>
    """
  end
end
