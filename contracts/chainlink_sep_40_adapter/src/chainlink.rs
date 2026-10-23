use soroban_sdk::{BytesN, Env, I256, contractclient, contracterror, contracttype};

/// Chainlink's Data Feeds proxy, as deployed on Stellar (`smartcontractkit/chainlink-stellar`).
#[allow(dead_code)]
#[contractclient(name = "DataFeedsProxyClient")]
pub trait DataFeedsProxy {
    fn latest_round(e: Env, data_id: BytesN<32>, decimals: u32) -> Result<Round, ProxyReadError>;
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Round {
    pub answer: I256,
    pub round_id: u64,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProxyReadError {
    NoDataPresent = 50,
    InvalidDecimals = 51,
    RoundsToZero = 52,
}
