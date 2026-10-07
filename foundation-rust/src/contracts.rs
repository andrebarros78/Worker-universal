use std::collections::BTreeMap;

use crate::model::Version;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDescriptor {
    pub id: String,
    pub version: Version,
    pub input_schema_ref: String,
    pub output_schema_ref: String,
}

#[derive(Debug, Default)]
pub struct ContractRegistry {
    contracts: BTreeMap<String, Vec<ContractDescriptor>>,
}

impl ContractRegistry {
    pub fn register(&mut self, contract: ContractDescriptor) -> Result<(), String> {
        if contract.id.trim().is_empty() {
            return Err("contract id cannot be empty".to_string());
        }
        let versions = self.contracts.entry(contract.id.clone()).or_default();
        if versions.iter().any(|item| item.version == contract.version) {
            return Err(format!(
                "duplicate contract version: {} {}",
                contract.id, contract.version
            ));
        }
        versions.push(contract);
        versions.sort_by_key(|item| std::cmp::Reverse(item.version));
        Ok(())
    }

    pub fn negotiate(&self, id: &str, required: Version) -> Option<&ContractDescriptor> {
        self.contracts
            .get(id)?
            .iter()
            .filter(|item| item.version.compatible_with(required))
            .max_by_key(|item| item.version)
    }

    pub fn versions(&self, id: &str) -> Vec<Version> {
        self.contracts
            .get(id)
            .map(|items| items.iter().map(|item| item.version).collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterManifest {
    pub adapter_id: String,
    pub adapter_version: Version,
    pub sdk_contract_version: Version,
    pub capabilities: Vec<String>,
    pub config_schema_ref: Option<String>,
    pub secret_refs: Vec<String>,
    pub simulation_supported: bool,
}

impl AdapterManifest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.adapter_id.trim().is_empty() {
            return Err("adapter_id cannot be empty");
        }
        if self.capabilities.is_empty() {
            return Err("adapter must expose at least one capability");
        }
        if self.sdk_contract_version.major == 0 {
            return Err("sdk contract major must be non-zero");
        }
        Ok(())
    }

    pub fn compatible_with_sdk(&self, required: Version) -> bool {
        self.sdk_contract_version.compatible_with(required)
    }
}
