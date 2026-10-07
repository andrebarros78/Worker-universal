defmodule TmaAvailability.Application do
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Task.Supervisor, name: TmaAvailability.TaskSupervisor}
    ]

    Supervisor.start_link(children,
      strategy: :one_for_one,
      name: TmaAvailability.Supervisor
    )
  end
end
