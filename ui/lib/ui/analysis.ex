defmodule Ui.Analysis do
  @moduledoc """
  Runs TINA's `sift` on the Petri net encoded from a BPMN choreography.

  The Rust NIF only produces the PNML document and the reachability formulas.
  The external process is started from Elixir on purpose: launching a child
  process inside a NIF is unreliable on the BEAM, which reaps the children
  before `wait` can observe them.

  Two properties are checked:

    * safety (`L.safe`), meaning that no place ever carries more than one
      token;
    * deadlock-freedom (`-L.dead \\/ (final)`), where the intended final
      marking, with all places empty, is accepted.

  `sift` always exits with status 0; the outcome is read from its output.
  """

  @safety_label "Safety (1-bounded)"
  @deadlock_label "Deadlock-freedom (final state allowed)"

  @doc """
  Encodes `bpmn`, checks it with the `sift` binary at `sift_path`, and returns
  a report map suitable for the UI.

  `max_states` and `timeout` bound the state-space exploration; pass `nil` to
  leave the corresponding `sift` limit unset.
  """
  def run(bpmn, sift_path, max_states \\ nil, timeout \\ nil) do
    with {:ok, json} <- Converter.analysis_inputs(bpmn),
         {:ok, inputs} <- decode_inputs(json) do
      analyze_pnml(inputs, sift_path, max_states, timeout)
    else
      {:error, reason} -> {:error, normalize_error(reason)}
    end
  end

  defp decode_inputs(json) do
    case Jason.decode(json) do
      {:ok,
       %{
         "pnml" => pnml,
         "safety_formula" => safety_formula,
         "deadlock_formula" => deadlock_formula,
         "places" => places,
         "transitions" => transitions
       }}
      when is_binary(pnml) and is_binary(safety_formula) and is_binary(deadlock_formula) and
             is_integer(places) and is_integer(transitions) ->
        {:ok,
         %{
           pnml: pnml,
           safety_formula: safety_formula,
           deadlock_formula: deadlock_formula,
           places: places,
           transitions: transitions
         }}

      _ ->
        {:error, "the encoder returned invalid analysis inputs"}
    end
  end

  defp analyze_pnml(inputs, sift_path, max_states, timeout) do
    path = temp_path()

    case File.write(path, inputs.pnml) do
      :ok ->
        try do
          with {:ok, safety} <-
                 check(
                   sift_path,
                   path,
                   inputs.safety_formula,
                   @safety_label,
                   max_states,
                   timeout
                 ),
               {:ok, deadlock} <-
                 check(
                   sift_path,
                   path,
                   inputs.deadlock_formula,
                   @deadlock_label,
                   max_states,
                   timeout
                 ) do
            {:ok,
             %{
               "ok" => true,
               "all_hold" => safety["holds"] and deadlock["holds"],
               "places" => inputs.places,
               "transitions" => inputs.transitions,
               "safety" => safety,
               "deadlock" => deadlock
             }}
          end
        after
          File.rm(path)
        end

      {:error, reason} ->
        {:error,
         "cannot create a temporary PNML file (#{Path.basename(path)}): " <>
           :file.format_error(reason)}
    end
  end

  defp check(sift_path, path, formula, property, max_states, timeout) do
    args =
      ["-q", "-PNML", "-f", formula]
      |> add_limit("-c", max_states)
      |> add_limit("-t", timeout)
      |> Kernel.++([path])

    with {:ok, output} <- run_sift(sift_path, args) do
      case classify(output) do
        :error ->
          {:error,
           "the Petri net analyzer '#{sift_path}' rejected the net or the formula: " <>
             reject_reason(output)}

        outcome ->
          {:ok,
           %{
             "property" => property,
             "status" => Atom.to_string(outcome),
             "holds" => outcome == :holds,
             "violation" => if(outcome == :violated, do: parse_violation(output)),
             "states" => parse_states(output),
             "log" => format_log(sift_path, args, output)
           }}
      end
    end
  end

  defp run_sift(sift_path, args) do
    case System.find_executable(sift_path) do
      nil ->
        {:error, missing_analyzer_message(sift_path)}

      executable ->
        try do
          {output, _status} = System.cmd(executable, args, stderr_to_stdout: true)
          {:ok, output}
        rescue
          error ->
            {:error,
             "could not run the Petri net analyzer '#{sift_path}': " <>
               Exception.message(error) <> missing_analyzer_hint()}
        end
    end
  end

  defp classify(output) do
    cond do
      String.contains?(output, "some state violates condition -f:") -> :violated
      String.contains?(output, "limit reached") -> :unknown
      Regex.match?(~r/\d+\s+marking\(s\)/, output) -> :holds
      true -> :error
    end
  end

  defp parse_states(output) do
    case Regex.run(~r/(\d+)\s+marking\(s\)/, output, capture: :all_but_first) do
      [states] -> String.to_integer(states)
      _ -> nil
    end
  end

  defp parse_violation(output) do
    lines = String.split(output, "\n")

    case Enum.find_index(lines, &String.contains?(&1, "some state violates condition")) do
      nil ->
        nil

      index ->
        details =
          lines
          |> Enum.drop(index + 1)
          |> Enum.take_while(&(not String.contains?(&1, "marking(s)")))
          |> Enum.map(&String.trim/1)
          |> Enum.reject(&(&1 == ""))

        case details do
          [] -> String.trim(Enum.at(lines, index))
          _ -> Enum.join(details, " | ")
        end
    end
  end

  defp add_limit(args, _flag, nil), do: args
  defp add_limit(args, _flag, value) when not is_integer(value) or value <= 0, do: args
  defp add_limit(args, flag, value), do: args ++ [flag, Integer.to_string(value)]

  defp format_log(sift_path, args, output) do
    command = Enum.map_join([sift_path | args], " ", &shell_quote/1)
    "$ #{command}\n" <> String.trim_trailing(output)
  end

  defp shell_quote(argument) do
    "'" <> String.replace(argument, "'", "'\\''") <> "'"
  end

  defp reject_reason(output) do
    case String.trim(output) do
      "" -> "no diagnostics produced"
      reason -> reason
    end
  end

  defp missing_analyzer_message(sift_path) do
    "the Petri net analyzer '#{sift_path}' was not found" <> missing_analyzer_hint()
  end

  defp missing_analyzer_hint do
    ". Install TINA (https://projects.laas.fr/tina/) or set SIFT_PATH to the 'sift' binary."
  end

  defp normalize_error(reason) when is_binary(reason), do: reason
  defp normalize_error(reason), do: inspect(reason)

  defp temp_path do
    Path.join(
      System.tmp_dir!(),
      "bpmner-analysis-#{System.unique_integer([:positive, :monotonic])}.pnml"
    )
  end
end
