from __future__ import annotations

from collections.abc import Callable, Iterable
from concurrent.futures import ThreadPoolExecutor, as_completed

from .models import MissionTask, RunResult
from .runner import TimedMissionRunner


class MissionOrchestrator:
    def __init__(self, runner_factory: Callable[[], TimedMissionRunner], max_workers: int = 4) -> None:
        if max_workers < 1:
            raise ValueError("max_workers must be >= 1")
        self.runner_factory = runner_factory
        self.max_workers = max_workers

    def run_many(self, tasks: Iterable[MissionTask]) -> list[RunResult]:
        indexed = list(enumerate(tasks))
        results: list[RunResult | None] = [None] * len(indexed)
        with ThreadPoolExecutor(max_workers=self.max_workers, thread_name_prefix="tma") as pool:
            futures = {pool.submit(self.runner_factory().run, task): index for index, task in indexed}
            for future in as_completed(futures):
                results[futures[future]] = future.result()
        return [result for result in results if result is not None]
