use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents any Asset on the Stellar Network or Soroban Environment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    /// Native XLM Asset
    Native,
    /// Alphanumeric 4-character Asset (e.g. USDC, AQUA)
    CreditAlphanum4 { code: String, issuer: String },
    /// Alphanumeric 12-character Asset (e.g. STELLAR_LUNC)
    CreditAlphanum12 { code: String, issuer: String },
    /// Soroban Smart Contract Token (SAC / Custom WASM)
    SorobanContract { address: String },
}

impl Asset {
    pub fn native() -> Self {
        Asset::Native
    }

    pub fn usdc(issuer: &str) -> Self {
        Asset::CreditAlphanum4 {
            code: "USDC".to_string(),
            issuer: issuer.to_string(),
        }
    }

    pub fn aqua(issuer: &str) -> Self {
        Asset::CreditAlphanum4 {
            code: "AQUA".to_string(),
            issuer: issuer.to_string(),
        }
    }

    pub fn yxlm(issuer: &str) -> Self {
        Asset::CreditAlphanum4 {
            code: "yXLM".to_string(),
            issuer: issuer.to_string(),
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Asset::Native => "XLM",
            Asset::CreditAlphanum4 { code, .. } => code.as_str(),
            Asset::CreditAlphanum12 { code, .. } => code.as_str(),
            Asset::SorobanContract { address } => &address[..8.min(address.len())],
        }
    }

    pub fn issuer(&self) -> Option<&str> {
        match self {
            Asset::Native => None,
            Asset::CreditAlphanum4 { issuer, .. } => Some(issuer.as_str()),
            Asset::CreditAlphanum12 { issuer, .. } => Some(issuer.as_str()),
            Asset::SorobanContract { address } => Some(address.as_str()),
        }
    }

    pub fn canonical_string(&self) -> String {
        match self {
            Asset::Native => "XLM".to_string(),
            Asset::CreditAlphanum4 { code, issuer } => format!("{}:{}", code, issuer),
            Asset::CreditAlphanum12 { code, issuer } => format!("{}:{}", code, issuer),
            Asset::SorobanContract { address } => format!("SOROBAN:{}", address),
        }
    }
}

impl fmt::Display for Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_string())
    }
}
