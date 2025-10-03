use crate::streaming::event_parser::protocols::{
    jupiter_agg_v6::{events::{JupiterAggV6RouteEvent, JupiterAggV6FeeEvent}, types::JupiterSwapEvent},
    raydium_amm_v4::events::RaydiumAmmV4SwapEvent,
    raydium_clmm::events::{RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event},
    raydium_cpmm::events::RaydiumCpmmSwapEvent,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a token pair for trading
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenPair {
    pub base: Pubkey,
    pub quote: Pubkey,
}

impl TokenPair {
    pub fn new(token_a: Pubkey, token_b: Pubkey) -> Self {
        // Normalize token pair ordering for consistent lookups
        if token_a.to_string() < token_b.to_string() {
            Self {
                base: token_a,
                quote: token_b,
            }
        } else {
            Self {
                base: token_b,
                quote: token_a,
            }
        }
    }

    pub fn is_reversed(&self, input: Pubkey, output: Pubkey) -> bool {
        input == self.quote && output == self.base
    }
}

/// Price information for a specific DEX
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceQuote {
    pub dex: DexType,
    pub token_pair: TokenPair,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price: f64, // output_amount / input_amount
    pub timestamp: u64,
    pub pool_address: Option<Pubkey>,
    pub slippage_bps: Option<u64>,
    pub platform_fee_bps: Option<u8>,
    pub total_fees: Option<u64>, // Total fees collected in output token
    pub signature: Option<String>,
}

impl PriceQuote {
    /// Calculate effective price considering direction
    pub fn effective_price(&self, is_reversed: bool) -> f64 {
        if is_reversed {
            1.0 / self.price
        } else {
            self.price
        }
    }

    /// Calculate net price after fees
    pub fn net_price(&self) -> f64 {
        let mut net_output = self.output_amount as f64;

        // Subtract platform fees
        if let Some(fee_bps) = self.platform_fee_bps {
            let fee_amount = (self.output_amount as f64 * fee_bps as f64) / 10000.0;
            net_output -= fee_amount;
        }

        // Subtract total fees if available
        if let Some(total_fees) = self.total_fees {
            net_output -= total_fees as f64;
        }

        net_output / self.input_amount as f64
    }

    /// Get estimated fee percentage
    pub fn estimated_fee_percentage(&self) -> f64 {
        if let Some(fee_bps) = self.platform_fee_bps {
            fee_bps as f64 / 100.0
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DexType {
    Jupiter,
    RaydiumAmmV4,
    RaydiumClmm,
    RaydiumCpmm,
}

/// Arbitrage opportunity detected between two DEXes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub token_pair: TokenPair,
    pub buy_dex: DexType,
    pub sell_dex: DexType,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit_percentage: f64,
    pub net_profit_percentage: f64, // Profit after fees
    pub timestamp: u64,
    pub buy_quote: PriceQuote,
    pub sell_quote: PriceQuote,
    pub total_fee_percentage: f64,
    pub estimated_gas_cost: f64, // Estimated in basis points
}

impl ArbitrageOpportunity {
    /// Calculate potential gross profit for a given input amount (before fees)
    pub fn calculate_profit(&self, input_amount: f64) -> f64 {
        let bought_amount = input_amount / self.buy_price;
        let sold_amount = bought_amount * self.sell_price;
        sold_amount - input_amount
    }

    /// Calculate net profit after fees for a given input amount
    pub fn calculate_net_profit(&self, input_amount: f64) -> f64 {
        let bought_amount = input_amount / self.buy_quote.net_price();
        let sold_amount = bought_amount * self.sell_quote.net_price();

        // Subtract estimated gas costs (assuming ~0.001 SOL per transaction, 2 transactions)
        let gas_cost_lamports = 2_000_000.0; // 0.002 SOL in lamports
        sold_amount - input_amount - gas_cost_lamports
    }

    /// Calculate gross profit percentage
    pub fn profit_percentage(&self) -> f64 {
        ((self.sell_price - self.buy_price) / self.buy_price) * 100.0
    }

    /// Check if opportunity is profitable after fees and gas
    pub fn is_profitable_after_fees(&self) -> bool {
        self.net_profit_percentage > 0.0
    }

    /// Get total estimated fee cost in percentage
    pub fn total_cost_percentage(&self) -> f64 {
        self.total_fee_percentage + self.estimated_gas_cost / 100.0
    }
}

/// Arbitrage detector that monitors prices across DEXes
pub struct ArbitrageDetector {
    /// Store recent price quotes by token pair
    price_cache: HashMap<TokenPair, Vec<PriceQuote>>,
    /// Store fee events by signature for correlation
    fee_cache: HashMap<String, Vec<FeeInfo>>,
    /// Minimum profit percentage threshold (net after fees)
    min_profit_threshold: f64,
    /// Maximum age of price quotes in seconds
    max_quote_age_secs: u64,
}

/// Fee information from transaction logs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeInfo {
    pub signature: String,
    pub account: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub timestamp: u64,
}

impl ArbitrageDetector {
    pub fn new(min_profit_threshold: f64, max_quote_age_secs: u64) -> Self {
        Self {
            price_cache: HashMap::new(),
            fee_cache: HashMap::new(),
            min_profit_threshold,
            max_quote_age_secs,
        }
    }

    /// Process fee event and associate with recent quotes
    pub fn process_fee_event(&mut self, event: &JupiterAggV6FeeEvent) {
        let signature = event.metadata.signature.to_string();
        let fee_info = FeeInfo {
            signature: signature.clone(),
            account: event.account,
            mint: event.mint,
            amount: event.amount,
            timestamp: Self::current_timestamp(),
        };

        self.fee_cache
            .entry(signature)
            .or_insert_with(Vec::new)
            .push(fee_info);

        // Clean old fee entries
        self.clean_old_fees();
    }

    /// Add Jupiter swap event to price cache
    pub fn process_jupiter_route(&mut self, event: &JupiterAggV6RouteEvent) -> Vec<ArbitrageOpportunity> {
        let token_pair = TokenPair::new(event.source_mint, event.destination_mint);
        let price = event.quoted_out_amount as f64 / event.in_amount as f64;

        // Check for associated fees
        let signature = event.metadata.signature.to_string();
        let total_fees = self.get_total_fees_for_signature(&signature, &event.destination_mint);

        let quote = PriceQuote {
            dex: DexType::Jupiter,
            token_pair: token_pair.clone(),
            input_amount: event.in_amount,
            output_amount: event.quoted_out_amount,
            price,
            timestamp: Self::current_timestamp(),
            pool_address: None,
            slippage_bps: Some(event.slippage_bps),
            platform_fee_bps: Some(event.platform_fee_bps),
            total_fees,
            signature: Some(signature),
        };

        self.add_price_quote(quote)
    }

    /// Add Jupiter swap event from logs
    pub fn process_jupiter_swap(&mut self, event: &JupiterSwapEvent) -> Vec<ArbitrageOpportunity> {
        let token_pair = TokenPair::new(event.input_mint, event.output_mint);
        let price = event.output_amount as f64 / event.input_amount as f64;

        let quote = PriceQuote {
            dex: DexType::Jupiter,
            token_pair: token_pair.clone(),
            input_amount: event.input_amount,
            output_amount: event.output_amount,
            price,
            timestamp: Self::current_timestamp(),
            pool_address: Some(event.amm),
            slippage_bps: None,
            platform_fee_bps: None,
            total_fees: None,
            signature: None,
        };

        self.add_price_quote(quote)
    }

    /// Add Raydium AMM V4 swap event
    ///
    /// WARNING: RaydiumAmmV4SwapEvent does NOT contain token mint fields!
    /// It only has pool_coin_token_account and pool_pc_token_account which are token ACCOUNTS, not MINTS.
    /// These will NOT match with other DEXes that use mints.
    ///
    /// TODO: Implement mint lookup from pool state or account metadata to enable AMM V4 arbitrage detection.
    pub fn process_raydium_amm_v4_swap(&mut self, _event: &RaydiumAmmV4SwapEvent) -> Vec<ArbitrageOpportunity> {
        // Skip AMM V4 events as they can't be matched properly with other DEXes
        // Return empty vec to avoid false arbitrage signals
        Vec::new()
    }

    /// Add Raydium CLMM swap event
    ///
    /// WARNING: RaydiumClmmSwapEvent uses vault addresses (input_vault, output_vault), not mints!
    /// This will NOT match with other DEXes that use mints.
    /// Use RaydiumClmmSwapV2Event instead which has input_vault_mint/output_vault_mint fields.
    pub fn process_raydium_clmm_swap(&mut self, _event: &RaydiumClmmSwapEvent) -> Vec<ArbitrageOpportunity> {
        // Skip CLMM v1 events as they can't be matched properly with other DEXes
        // Return empty vec to avoid false arbitrage signals
        Vec::new()
    }

    /// Add Raydium CLMM V2 swap event
    pub fn process_raydium_clmm_swap_v2(&mut self, event: &RaydiumClmmSwapV2Event) -> Vec<ArbitrageOpportunity> {
        // FIXED: Use the actual token mints instead of vault addresses
        let token_pair = TokenPair::new(event.input_vault_mint, event.output_vault_mint);

        let price = event.other_amount_threshold as f64 / event.amount as f64;

        let quote = PriceQuote {
            dex: DexType::RaydiumClmm,
            token_pair: token_pair.clone(),
            input_amount: event.amount,
            output_amount: event.other_amount_threshold,
            price,
            timestamp: Self::current_timestamp(),
            pool_address: Some(event.pool_state),
            slippage_bps: None,
            platform_fee_bps: None,
            total_fees: None,
            signature: None,
        };

        self.add_price_quote(quote)
    }

    /// Add Raydium CPMM swap event
    pub fn process_raydium_cpmm_swap(&mut self, event: &RaydiumCpmmSwapEvent) -> Vec<ArbitrageOpportunity> {
        let token_pair = TokenPair::new(event.input_token_mint, event.output_token_mint);

        let (input_amount, output_amount) = if event.amount_in > 0 {
            (event.amount_in, event.minimum_amount_out)
        } else {
            (event.max_amount_in, event.amount_out)
        };

        let price = output_amount as f64 / input_amount as f64;

        let quote = PriceQuote {
            dex: DexType::RaydiumCpmm,
            token_pair: token_pair.clone(),
            input_amount,
            output_amount,
            price,
            timestamp: Self::current_timestamp(),
            pool_address: Some(event.pool_state),
            slippage_bps: None,
            platform_fee_bps: None,
            total_fees: None,
            signature: None,
        };

        self.add_price_quote(quote)
    }

    /// Get total fees for a given signature and mint
    fn get_total_fees_for_signature(&self, signature: &str, mint: &Pubkey) -> Option<u64> {
        self.fee_cache.get(signature).and_then(|fees| {
            let total: u64 = fees
                .iter()
                .filter(|fee| &fee.mint == mint)
                .map(|fee| fee.amount)
                .sum();
            if total > 0 {
                Some(total)
            } else {
                None
            }
        })
    }

    /// Clean old fee entries
    fn clean_old_fees(&mut self) {
        let now = Self::current_timestamp();
        let max_age = self.max_quote_age_secs;

        self.fee_cache.retain(|_, fees| {
            fees.retain(|f| now - f.timestamp <= max_age);
            !fees.is_empty()
        });
    }

    /// Add a price quote and check for arbitrage opportunities
    fn add_price_quote(&mut self, quote: PriceQuote) -> Vec<ArbitrageOpportunity> {
        let token_pair = quote.token_pair.clone();

        // Clean old quotes
        self.clean_old_quotes();

        // Add quote to cache
        let quotes_for_pair = self.price_cache
            .entry(token_pair.clone())
            .or_insert_with(Vec::new);

        println!("📊 Adding quote: {:?} - {}/{} = {:.6} (existing quotes: {})",
            quote.dex,
            quote.output_amount,
            quote.input_amount,
            quote.price,
            quotes_for_pair.len()
        );

        quotes_for_pair.push(quote.clone());

        // Find arbitrage opportunities
        self.find_arbitrage_opportunities(&token_pair)
    }

    /// Find arbitrage opportunities for a token pair
    fn find_arbitrage_opportunities(&self, token_pair: &TokenPair) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();

        if let Some(quotes) = self.price_cache.get(token_pair) {
            println!("🔍 Checking {} quotes for token pair {}/{}",
                quotes.len(),
                token_pair.base,
                token_pair.quote
            );

            // Compare each pair of quotes from different DEXes
            for (i, quote1) in quotes.iter().enumerate() {
                for quote2 in quotes.iter().skip(i + 1) {
                    // Only compare quotes from different DEXes
                    if quote1.dex == quote2.dex {
                        continue;
                    }

                    // Check if quotes are recent enough
                    let now = Self::current_timestamp();
                    if now - quote1.timestamp > self.max_quote_age_secs
                        || now - quote2.timestamp > self.max_quote_age_secs
                    {
                        println!("⏰ Quotes too old: age1={}, age2={} (max={})",
                            now - quote1.timestamp,
                            now - quote2.timestamp,
                            self.max_quote_age_secs
                        );
                        continue;
                    }

                    // Calculate profit potential
                    if let Some(opportunity) = self.calculate_arbitrage(quote1, quote2) {
                        println!("💡 Found price difference: {:?} @ {:.6} vs {:?} @ {:.6} = {:.2}% profit",
                            quote1.dex,
                            quote1.price,
                            quote2.dex,
                            quote2.price,
                            opportunity.profit_percentage
                        );

                        if opportunity.profit_percentage >= self.min_profit_threshold {
                            println!("✅ Meets threshold! Adding opportunity");
                            opportunities.push(opportunity);
                        } else {
                            println!("❌ Below {:.2}% threshold", self.min_profit_threshold);
                        }
                    }
                }
            }
        } else {
            println!("📭 No quotes found for this token pair yet");
        }

        opportunities
    }

    /// Calculate arbitrage opportunity between two quotes
    fn calculate_arbitrage(
        &self,
        quote1: &PriceQuote,
        quote2: &PriceQuote,
    ) -> Option<ArbitrageOpportunity> {
        // Determine which is buy and which is sell based on prices
        let (buy_quote, sell_quote) = if quote1.price < quote2.price {
            (quote1, quote2)
        } else {
            (quote2, quote1)
        };

        // Calculate gross profit percentage
        let profit_pct = ((sell_quote.price - buy_quote.price) / buy_quote.price) * 100.0;

        // Calculate net profit percentage after fees
        let buy_net_price = buy_quote.net_price();
        let sell_net_price = sell_quote.net_price();
        let net_profit_pct = ((sell_net_price - buy_net_price) / buy_net_price) * 100.0;

        // Calculate total fee percentage
        let total_fee_pct = buy_quote.estimated_fee_percentage() + sell_quote.estimated_fee_percentage();

        // Estimate gas cost (approximately 0.001 SOL per transaction * 2 = 0.002 SOL)
        // As percentage of a typical 1 SOL transaction = 0.2%
        let estimated_gas_cost = 20.0; // in basis points (0.2%)

        Some(ArbitrageOpportunity {
            token_pair: quote1.token_pair.clone(),
            buy_dex: buy_quote.dex.clone(),
            sell_dex: sell_quote.dex.clone(),
            buy_price: buy_quote.price,
            sell_price: sell_quote.price,
            profit_percentage: profit_pct,
            net_profit_percentage: net_profit_pct,
            timestamp: Self::current_timestamp(),
            buy_quote: buy_quote.clone(),
            sell_quote: sell_quote.clone(),
            total_fee_percentage: total_fee_pct,
            estimated_gas_cost,
        })
    }

    /// Clean quotes older than max age
    fn clean_old_quotes(&mut self) {
        let now = Self::current_timestamp();
        let max_age = self.max_quote_age_secs;

        self.price_cache.retain(|_, quotes| {
            quotes.retain(|q| now - q.timestamp <= max_age);
            !quotes.is_empty()
        });
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get current price quotes for a token pair
    pub fn get_quotes(&self, token_pair: &TokenPair) -> Option<&Vec<PriceQuote>> {
        self.price_cache.get(token_pair)
    }

    /// Get all token pairs being tracked
    pub fn get_tracked_pairs(&self) -> Vec<TokenPair> {
        self.price_cache.keys().cloned().collect()
    }

    /// Clear all cached quotes
    pub fn clear(&mut self) {
        self.price_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_pair_normalization() {
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();

        let pair1 = TokenPair::new(pubkey1, pubkey2);
        let pair2 = TokenPair::new(pubkey2, pubkey1);

        assert_eq!(pair1, pair2);
    }

    #[test]
    fn test_arbitrage_detection() {
        let mut detector = ArbitrageDetector::new(0.5, 60);

        let token_a = Pubkey::new_unique();
        let token_b = Pubkey::new_unique();
        let token_pair = TokenPair::new(token_a, token_b);

        // Add Jupiter quote
        let jupiter_quote = PriceQuote {
            dex: DexType::Jupiter,
            token_pair: token_pair.clone(),
            input_amount: 1000,
            output_amount: 1100, // Price: 1.1
            price: 1.1,
            timestamp: ArbitrageDetector::current_timestamp(),
            pool_address: None,
            slippage_bps: None,
            platform_fee_bps: None,
            total_fees: None,
            signature: None,
        };

        // Add Raydium quote with higher price
        let raydium_quote = PriceQuote {
            dex: DexType::RaydiumCpmm,
            token_pair: token_pair.clone(),
            input_amount: 1000,
            output_amount: 1150, // Price: 1.15
            price: 1.15,
            timestamp: ArbitrageDetector::current_timestamp(),
            pool_address: None,
            slippage_bps: None,
            platform_fee_bps: None,
            total_fees: None,
            signature: None,
        };

        let opps1 = detector.add_price_quote(jupiter_quote);
        assert_eq!(opps1.len(), 0); // No opportunity yet

        let opps2 = detector.add_price_quote(raydium_quote);
        assert_eq!(opps2.len(), 1); // Should find 1 opportunity

        let opp = &opps2[0];
        assert_eq!(opp.buy_dex, DexType::Jupiter);
        assert_eq!(opp.sell_dex, DexType::RaydiumCpmm);
        assert!(opp.profit_percentage > 4.0 && opp.profit_percentage < 5.0);
    }
}
