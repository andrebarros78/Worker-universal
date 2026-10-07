defmodule TmaAvailability.RetryTest do
  use ExUnit.Case, async: true

  test "recovers within restart budget" do
    {:ok, agent} = Agent.start_link(fn -> 0 end)

    result =
      TmaAvailability.Retry.run(
        fn ->
          call = Agent.get_and_update(agent, fn value -> {value + 1, value + 1} end)

          if call < 3 do
            {:error, :synthetic}
          else
            :ok
          end
        end,
        3
      )

    assert result == {:ok, 3}
  end

  test "returns final failure after budget" do
    assert {:error, :nope, 2} =
             TmaAvailability.Retry.run(fn -> {:error, :nope} end, 1)
  end
end
