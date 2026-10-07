defmodule TmaAvailability.MixProject do
  use Mix.Project

  def project do
    [
      app: :tma_availability,
      version: "0.2.0",
      elixir: "~> 1.20",
      start_permanent: Mix.env() == :prod,
      deps: []
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {TmaAvailability.Application, []}
    ]
  end
end
