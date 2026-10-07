from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Protocol


@dataclass(frozen=True, slots=True)
class AdapterManifest:
    adapter_id: str
    adapter_version: str
    sdk_contract_version: str
    capabilities: tuple[str, ...]
    simulation_supported: bool
    config_schema_ref: str | None = None
    secret_refs: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class Invocation:
    invocation_id: str
    mission_id: str
    capability_id: str
    contract_version: str
    timeout_ms: int
    payload: Any
    simulation: bool = False
    idempotency_key: str | None = None
    session_ref: str | None = None
    secret_refs: tuple[str, ...] = ()
    artifact_refs: tuple[str, ...] = ()


@dataclass(slots=True)
class AdapterResult:
    status: str
    payload: Any = None
    errors: list[dict[str, Any]] = field(default_factory=list)
    evidence: list[dict[str, Any]] = field(default_factory=list)
    artifact_refs: list[str] = field(default_factory=list)
    session_ref: str | None = None
    cost_microunits: int = 0


class Adapter(Protocol):
    def manifest(self) -> AdapterManifest: ...

    def health(self) -> tuple[str, str]: ...

    def invoke(self, invocation: Invocation) -> AdapterResult: ...


def validate_manifest(manifest: AdapterManifest) -> None:
    if not manifest.adapter_id.strip():
        raise ValueError("adapter_id required")
    if not manifest.capabilities:
        raise ValueError("at least one capability required")
    if not manifest.sdk_contract_version.startswith("1."):
        raise ValueError("unsupported sdk contract major")
