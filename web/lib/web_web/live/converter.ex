defmodule Converter do
  use Rustler, otp_app: :web, crate: "nif"

  def convert_bpmn_to_pnml(_), do: :erlang.nif_error(:nif_not_loaded)
  def convert_bpmn_to_dot(_), do: :erlang.nif_error(:nif_not_loaded)
end
