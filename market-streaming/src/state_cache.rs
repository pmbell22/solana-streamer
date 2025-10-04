use crate::pool_states::DexPoolState;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;

/// Cached pool state with metadata
#[derive(Clone, Debug)]
pub struct CachedPoolState {
    /// The pool state
    pub state: DexPoolState,
    /// Slot when this state was last updated
    pub slot: u64,
    /// Timestamp when this state was cached (in milliseconds)
    pub cached_at: u64,
}

impl CachedPoolState {
    pub fn new(state: DexPoolState, slot: u64) -> Self {
        Self {
            state,
            slot,
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Check if the cached state is stale (older than specified milliseconds)
    pub fn is_stale(&self, max_age_ms: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now - self.cached_at > max_age_ms
    }
}

/// Thread-safe cache for pool states
pub struct PoolStateCache {
    /// Map of pool pubkey to cached state
    cache: Arc<DashMap<Pubkey, CachedPoolState>>,
    /// Maximum age of cached states in milliseconds (default: 5000ms)
    max_age_ms: u64,
}

impl PoolStateCache {
    /// Create a new pool state cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_age_ms: 5000, // 5 seconds default
        }
    }

    /// Create a new pool state cache with custom max age
    pub fn with_max_age(max_age_ms: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_age_ms,
        }
    }

    /// Update a pool state
    pub fn update(&self, pubkey: Pubkey, state: DexPoolState, slot: u64) {
        self.cache
            .insert(pubkey, CachedPoolState::new(state, slot));
    }

    /// Get a pool state
    pub fn get(&self, pubkey: &Pubkey) -> Option<CachedPoolState> {
        self.cache.get(pubkey).map(|entry| entry.value().clone())
    }

    /// Get a pool state only if it's not stale
    pub fn get_fresh(&self, pubkey: &Pubkey) -> Option<CachedPoolState> {
        self.cache.get(pubkey).and_then(|entry| {
            let cached = entry.value();
            if !cached.is_stale(self.max_age_ms) {
                Some(cached.clone())
            } else {
                None
            }
        })
    }

    /// Remove a pool from cache
    pub fn remove(&self, pubkey: &Pubkey) -> Option<CachedPoolState> {
        self.cache.remove(pubkey).map(|(_, v)| v)
    }

    /// Clear all cached states
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get all pool pubkeys currently in cache
    pub fn get_all_pubkeys(&self) -> Vec<Pubkey> {
        self.cache.iter().map(|entry| *entry.key()).collect()
    }

    /// Get number of cached pools
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get all fresh pool states
    pub fn get_all_fresh(&self) -> Vec<(Pubkey, CachedPoolState)> {
        self.cache
            .iter()
            .filter_map(|entry| {
                let cached = entry.value();
                if !cached.is_stale(self.max_age_ms) {
                    Some((*entry.key(), cached.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Remove all stale entries from cache
    pub fn cleanup_stale(&self) {
        let stale_keys: Vec<Pubkey> = self
            .cache
            .iter()
            .filter_map(|entry| {
                if entry.value().is_stale(self.max_age_ms) {
                    Some(*entry.key())
                } else {
                    None
                }
            })
            .collect();

        for key in stale_keys {
            self.cache.remove(&key);
        }
    }

    /// Get statistics about the cache
    pub fn stats(&self) -> CacheStats {
        let total = self.cache.len();
        let fresh = self.get_all_fresh().len();
        let stale = total - fresh;

        CacheStats {
            total_entries: total,
            fresh_entries: fresh,
            stale_entries: stale,
            max_age_ms: self.max_age_ms,
        }
    }
}

impl Default for PoolStateCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub fresh_entries: usize,
    pub stale_entries: usize,
    pub max_age_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pool_states::{DexPoolState, RaydiumClmmPoolState};

    #[test]
    fn test_cache_basic_operations() {
        let cache = PoolStateCache::new();
        let pubkey = Pubkey::new_unique();

        // Create a dummy pool state
        let pool_state = DexPoolState::RaydiumClmm(RaydiumClmmPoolState {
            bump: [0],
            amm_config: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            token_mint_0: Pubkey::new_unique(),
            token_mint_1: Pubkey::new_unique(),
            token_vault_0: Pubkey::new_unique(),
            token_vault_1: Pubkey::new_unique(),
            observation_key: Pubkey::new_unique(),
            mint_decimals_0: 9,
            mint_decimals_1: 6,
            tick_spacing: 1,
            liquidity: 1000000,
            sqrt_price_x64: 1 << 64,
            tick_current: 0,
            padding3: 0,
            padding4: 0,
            fee_growth_global_0_x64: 0,
            fee_growth_global_1_x64: 0,
            protocol_fees_token_0: 0,
            protocol_fees_token_1: 0,
            swap_in_amount_token_0: 0,
            swap_out_amount_token_1: 0,
            swap_in_amount_token_1: 0,
            swap_out_amount_token_0: 0,
            status: 0,
            padding: [0; 7],
            recent_epoch: 0,
        });

        // Test insert and get
        cache.update(pubkey, pool_state.clone(), 12345);
        assert_eq!(cache.len(), 1);

        let cached = cache.get(&pubkey).unwrap();
        assert_eq!(cached.slot, 12345);

        // Test remove
        cache.remove(&pubkey);
        assert!(cache.is_empty());
    }
}
