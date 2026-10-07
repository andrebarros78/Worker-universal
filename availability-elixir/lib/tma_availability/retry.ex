defmodule TmaAvailability.Retry do
  @spec run((-> :ok | {:error, term()}), non_neg_integer()) ::
          {:ok, non_neg_integer()} | {:error, term(), non_neg_integer()}
  def run(fun, max_restarts) when is_function(fun, 0) and max_restarts >= 0 do
    do_run(fun, max_restarts, 0)
  end

  defp do_run(fun, remaining, attempts) do
    case fun.() do
      :ok ->
        {:ok, attempts + 1}

      {:error, _reason} when remaining > 0 ->
        do_run(fun, remaining - 1, attempts + 1)

      {:error, reason} ->
        {:error, reason, attempts + 1}
    end
  end
end
