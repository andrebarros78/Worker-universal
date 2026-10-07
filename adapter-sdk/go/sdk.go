package adaptersdk

import (
	"context"
	"errors"
	"strings"
)

type AdapterManifest struct {
	AdapterID           string
	AdapterVersion      string
	SDKContractVersion  string
	Capabilities        []string
	SimulationSupported bool
	ConfigSchemaRef     string
	SecretRefs          []string
}

type Invocation struct {
	InvocationID    string
	MissionID       string
	CapabilityID    string
	ContractVersion string
	TimeoutMS       uint64
	Payload         any
	Simulation      bool
	IdempotencyKey  string
	SessionRef      string
	SecretRefs      []string
	ArtifactRefs    []string
}

type AdapterResult struct {
	Status         string
	Payload        any
	Errors         []map[string]any
	Evidence       []map[string]any
	ArtifactRefs   []string
	SessionRef     string
	CostMicrounits uint64
}

type Adapter interface {
	Manifest() AdapterManifest
	Health(context.Context) (lifecycle string, readiness string, err error)
	Invoke(context.Context, Invocation) AdapterResult
}

func ValidateManifest(manifest AdapterManifest) error {
	if strings.TrimSpace(manifest.AdapterID) == "" {
		return errors.New("adapter id required")
	}
	if len(manifest.Capabilities) == 0 {
		return errors.New("at least one capability required")
	}
	if !strings.HasPrefix(manifest.SDKContractVersion, "1.") {
		return errors.New("unsupported sdk contract major")
	}
	return nil
}
