defmodule Ui.Examples do
  @moduledoc """
  BPMN models bundled with the web interface.

  The files live in `priv/examples` so that they are included in releases.
  """

  @examples [
    %{
      id: "pizzeria",
      name: "Pizzeria (AND gateways)",
      description: "A small choreography with parallel gateways and three participants.",
      file: "pizzeria.bpmn"
    },
    %{
      id: "paper-example",
      name: "Paper example (XOR / OR gateways)",
      description: "The example from the paper, exercising XOR and OR gateways.",
      file: "paper-example.bpmn"
    }
  ]

  def all, do: @examples

  def get(id) do
    Enum.find(@examples, &(&1.id == id))
  end

  @doc "Reads the BPMN XML of an example."
  def read(%{file: file}) do
    case File.read(Path.join([priv_dir(), "examples", file])) do
      {:ok, xml} -> {:ok, xml}
      {:error, reason} -> {:error, "cannot read example '#{file}': #{:file.format_error(reason)}"}
    end
  end

  def read(_), do: {:error, "unknown example"}

  defp priv_dir do
    case :code.priv_dir(:ui) do
      {:error, _} -> Path.expand("../../priv", __DIR__)
      dir -> to_string(dir)
    end
  end
end
