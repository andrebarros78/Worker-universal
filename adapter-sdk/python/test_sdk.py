import unittest

from sdk import AdapterManifest, validate_manifest


class AdapterSdkTests(unittest.TestCase):
    def test_valid_manifest(self):
        manifest = AdapterManifest(
            adapter_id="local.echo",
            adapter_version="1.0.0",
            sdk_contract_version="1.0.0",
            capabilities=("native.local.echo",),
            simulation_supported=True,
        )
        validate_manifest(manifest)

    def test_rejects_empty_capability_list(self):
        manifest = AdapterManifest(
            adapter_id="bad",
            adapter_version="1.0.0",
            sdk_contract_version="1.0.0",
            capabilities=(),
            simulation_supported=False,
        )
        with self.assertRaises(ValueError):
            validate_manifest(manifest)


if __name__ == "__main__":
    unittest.main()
