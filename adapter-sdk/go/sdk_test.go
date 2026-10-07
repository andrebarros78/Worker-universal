package adaptersdk

import "testing"

func TestValidateManifest(t *testing.T) {
	err := ValidateManifest(AdapterManifest{
		AdapterID:           "local.echo",
		AdapterVersion:      "1.0.0",
		SDKContractVersion:  "1.0.0",
		Capabilities:        []string{"native.local.echo"},
		SimulationSupported: true,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestValidateManifestRejectsEmptyCapabilities(t *testing.T) {
	err := ValidateManifest(AdapterManifest{
		AdapterID:          "bad",
		AdapterVersion:     "1.0.0",
		SDKContractVersion: "1.0.0",
	})
	if err == nil {
		t.Fatal("expected error")
	}
}
