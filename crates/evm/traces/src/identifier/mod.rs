use alloy_json_abi::JsonAbi;
use alloy_primitives::{Address, Bytes, map::HashMap};
use foundry_common::ContractsByArtifact;
use foundry_compilers::ArtifactId;
use foundry_config::{Chain, Config};
use revm_inspectors::tracing::types::CallTraceNode;
use std::borrow::Cow;

mod local;
pub use local::LocalTraceIdentifier;

mod etherscan;
pub use etherscan::EtherscanIdentifier;

mod signatures;
pub use signatures::{SignaturesCache, SignaturesIdentifier};

/// An address identified by a [`TraceIdentifier`].
pub struct IdentifiedAddress<'a> {
    /// The address.
    pub address: Address,
    /// The label for the address.
    pub label: Option<String>,
    /// The contract this address represents.
    ///
    /// Note: This may be in the format `"<artifact>:<contract>"`.
    pub contract: Option<String>,
    /// The ABI of the contract at this address.
    pub abi: Option<Cow<'a, JsonAbi>>,
    /// The artifact ID of the contract, if any.
    pub artifact_id: Option<ArtifactId>,
}

/// Trace identifiers figure out what ABIs and labels belong to all the addresses of the trace.
pub trait TraceIdentifier {
    /// Attempts to identify an address in one or more call traces.
    fn identify_addresses(&mut self, nodes: &[&CallTraceNode]) -> Vec<IdentifiedAddress<'_>>;
}

/// A collection of trace identifiers.
pub struct TraceIdentifiers<'a> {
    /// The local trace identifier.
    pub local: Option<LocalTraceIdentifier<'a>>,
    /// The optional Etherscan trace identifier.
    pub etherscan: Option<EtherscanIdentifier>,
}

impl Default for TraceIdentifiers<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceIdentifier for TraceIdentifiers<'_> {
    fn identify_addresses(&mut self, nodes: &[&CallTraceNode]) -> Vec<IdentifiedAddress<'_>> {
        let mut identities = Vec::with_capacity(nodes.len());
        if let Some(local) = &mut self.local {
            identities.extend(local.identify_addresses(nodes));
            if identities.len() >= nodes.len() {
                return identities;
            }
        }
        if let Some(etherscan) = &mut self.etherscan {
            identities.extend(etherscan.identify_addresses(nodes));
        }
        identities
    }
}

impl<'a> TraceIdentifiers<'a> {
    /// Creates a new, empty instance.
    pub const fn new() -> Self {
        Self { local: None, etherscan: None }
    }

    /// Sets the local identifier.
    pub fn with_local(mut self, known_contracts: &'a ContractsByArtifact) -> Self {
        self.local = Some(LocalTraceIdentifier::new(known_contracts));
        self
    }

    /// Sets the local identifier.
    pub fn with_local_and_bytecodes(
        mut self,
        known_contracts: &'a ContractsByArtifact,
        contracts_bytecode: &'a HashMap<Address, Bytes>,
    ) -> Self {
        self.local =
            Some(LocalTraceIdentifier::new(known_contracts).with_bytecodes(contracts_bytecode));
        self
    }

    /// Sets the etherscan identifier.
    pub fn with_etherscan(mut self, config: &Config, chain: Option<Chain>) -> eyre::Result<Self> {
        self.etherscan = EtherscanIdentifier::new(config, chain)?;
        Ok(self)
    }

    /// Returns `true` if there are no set identifiers.
    pub fn is_empty(&self) -> bool {
        self.local.is_none() && self.etherscan.is_none()
    }
}
