defmodule Converter do
  @moduledoc """
  Thin Elixir wrapper around the Rust conversion and analysis NIF.
  """

  use Rustler, otp_app: :ui, crate: "nif"

  @doc "Encodes a BPMN choreography into PNML."
  def convert_bpmn_to_pnml(_bpmn, _initial_marking), do: :erlang.nif_error(:nif_not_loaded)

  @doc "Encodes a BPMN choreography into Graphviz DOT."
  def convert_bpmn_to_dot(_bpmn, _initial_marking, _rankdir_lr),
    do: :erlang.nif_error(:nif_not_loaded)

  @doc """
  Encodes the choreography and returns the inputs used for the external
  analysis: the PNML document, the safety/deadlock formulas, and the net size.
  """
  def analysis_inputs(_bpmn), do: :erlang.nif_error(:nif_not_loaded)
end
