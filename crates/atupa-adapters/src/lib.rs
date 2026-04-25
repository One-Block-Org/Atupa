pub trait ProtocolAdapter {
    /// The name of the protocol (e.g., "Uniswap v4").
    fn name(&self) -> &str;

    /// Resolves a combination of target address and function selector into a human-readable label.
    fn resolve_label(&self, address: Option<&str>, selector: Option<&str>) -> Option<String>;
}

/// Adapter specifically for identifying Uniswap v4 Hooks
pub struct UniswapV4Adapter;

impl ProtocolAdapter for UniswapV4Adapter {
    fn name(&self) -> &str {
        "Uniswap v4"
    }

    fn resolve_label(&self, _address: Option<&str>, selector: Option<&str>) -> Option<String> {
        let sel = selector?;
        // Uniswap v4 Hook standard interface selectors
        let label = match sel {
            "0x18a9d381" => "beforeInitialize",
            "0x999dea5d" => "afterInitialize",
            "0x910746f2" => "beforeAddLiquidity",
            "0xefd81287" => "afterAddLiquidity",
            "0xd7386be3" => "beforeRemoveLiquidity",
            "0x1efe5f9e" => "afterRemoveLiquidity",
            "0xe82c3b75" => "beforeSwap",
            "0x14d6eaec" => "afterSwap",
            "0xa3d03227" => "beforeDonate",
            "0x0df2d576" => "afterDonate",
            _ => return None,
        };

        Some(format!("Uniswapv4: {}", label))
    }
}

/// Adapter specifically for identifying Aave v3 Pool operations
pub struct AaveV3Adapter;

impl ProtocolAdapter for AaveV3Adapter {
    fn name(&self) -> &str {
        "Aave v3"
    }

    fn resolve_label(&self, _address: Option<&str>, selector: Option<&str>) -> Option<String> {
        let sel = selector?;
        // Aave v3 Pool interface selectors
        let label = match sel {
            "0x617ba037" => "supply",
            "0x69328dec" => "withdraw",
            "0xa415bcad" => "borrow",
            "0x573ade81" => "repay",
            "0x00a718a9" => "liquidationCall",
            "0xab9c4b5d" => "flashLoan",
            "0x42b0b77c" => "flashLoanSimple",
            _ => return None,
        };

        Some(format!("Aave: {}", label))
    }
}

/// Adapter specifically for identifying Lido stETH operations
pub struct LidoAdapter;

impl ProtocolAdapter for LidoAdapter {
    fn name(&self) -> &str {
        "Lido stETH"
    }

    fn resolve_label(&self, address: Option<&str>, selector: Option<&str>) -> Option<String> {
        // Known Lido protocol contract addresses (Mainnet)
        const LIDO_ADDRESSES: &[(&str, &str)] = &[
            (
                "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
                "stETH (Lido Core)",
            ),
            (
                "0x55032650b14df07b85bF18A3a3eC8E0Af2e028d5",
                "NodeOperatorsRegistry",
            ),
            ("0x442af752419395f27ed54A848524a30028962bb2", "LidoOracle"),
            (
                "0x889edC2Bf57978ed079b851D273218ee42a2b349",
                "WithdrawalQueue",
            ),
            ("0x852f970761d74367f33B6C2e309a29D681E2F16a", "LegacyOracle"),
            ("0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", "wstETH"),
        ];

        if let Some(addr) = address {
            for &(known_addr, name) in LIDO_ADDRESSES {
                if addr.to_lowercase() == known_addr.to_lowercase() {
                    return Some(format!("Lido::{}", name));
                }
            }
        }

        let sel = selector?;
        // Selectors for major Lido protocol operations
        const LIDO_SELECTORS: &[(&str, &str)] = &[
            ("0xa1903eab", "submit"),
            ("0xea598cb0", "requestWithdrawals"),
            ("0x826a73d6", "requestWithdrawalsWithPermit"),
            ("0xe35ea9a5", "claimWithdrawals"),
            ("0x8b6ca260", "handleOracleReport"),
            ("0x39ba163b", "transferShares"),
            ("0x4dbcaef1", "transferSharesFrom"),
            ("0xa9059cbb", "transfer"),
            ("0x095ea7b3", "approve"),
            ("0x0a19ea81", "wrap"),
            ("0x1dfab2e1", "unwrap"),
        ];

        for &(known_sel, label) in LIDO_SELECTORS {
            if sel.contains(known_sel) {
                return Some(format!("stETH::{label}"));
            }
        }
        None
    }
}

/// The registry holding all known protocol adapters.
pub struct AdapterRegistry {
    adapters: Vec<Box<dyn ProtocolAdapter>>,
}

impl AdapterRegistry {
    /// Initialize a new registry pre-loaded with all supported adapters.
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: Vec::new(),
        };
        registry.register(Box::new(UniswapV4Adapter));
        registry.register(Box::new(AaveV3Adapter));
        registry.register(Box::new(LidoAdapter));
        registry
    }

    /// Register a custom adapter
    pub fn register(&mut self, adapter: Box<dyn ProtocolAdapter>) {
        self.adapters.push(adapter);
    }

    /// Iterates through adapters to find a descriptive label for the call.
    pub fn resolve(&self, address: Option<&str>, selector: Option<&str>) -> Option<String> {
        for adapter in &self.adapters {
            if let Some(label) = adapter.resolve_label(address, selector) {
                return Some(label);
            }
        }
        None
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
