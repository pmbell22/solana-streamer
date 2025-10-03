use solana_program::hash::hash;

/// Calculate an Anchor instruction discriminator
///
/// Anchor instruction discriminators are the first 8 bytes of SHA256("global:instruction_name")
///
/// # Example
/// ```
/// let disc = instruction_discriminator("swapBaseInput");
/// ```
pub fn instruction_discriminator(name: &str) -> [u8; 8] {
    discriminator("global", name)
}

/// Calculate an Anchor event discriminator
///
/// Anchor event discriminators are the first 8 bytes of SHA256("event:event_name")
///
/// # Example
/// ```
/// let disc = event_discriminator("SwapEvent");
/// ```
pub fn event_discriminator(name: &str) -> [u8; 8] {
    discriminator("event", name)
}

/// Calculate an Anchor account discriminator
///
/// Anchor account discriminators are the first 8 bytes of SHA256("account:account_name")
///
/// # Example
/// ```
/// let disc = account_discriminator("AmmConfig");
/// ```
pub fn account_discriminator(name: &str) -> [u8; 8] {
    discriminator("account", name)
}

/// Generic discriminator calculation
///
/// Calculates the first 8 bytes of SHA256("namespace:name")
fn discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let hash_result = hash(preimage.as_bytes());
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash_result.to_bytes()[..8]);
    discriminator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_discriminator() {
        // Test Jupiter route instruction
        let disc = instruction_discriminator("route");
        assert_eq!(disc, [229, 23, 203, 151, 122, 227, 173, 42]);
    }

    #[test]
    fn test_event_discriminator() {
        // Test Jupiter SwapEvent
        let disc = event_discriminator("SwapEvent");
        assert_eq!(disc, [64, 198, 205, 232, 38, 8, 113, 226]);

        // Test Jupiter FeeEvent
        let disc = event_discriminator("FeeEvent");
        assert_eq!(disc, [73, 79, 78, 127, 184, 213, 13, 220]);
    }

    #[test]
    fn test_account_discriminator() {
        // Test Raydium AmmConfig
        let disc = account_discriminator("AmmConfig");
        // This would need the actual expected value from the IDL
        println!("AmmConfig discriminator: {:?}", disc);
    }

    #[test]
    fn test_raydium_instructions() {
        // Test all Raydium CPMM instructions
        println!("swapBaseInput: {:?}", instruction_discriminator("swapBaseInput"));
        println!("swapBaseOutput: {:?}", instruction_discriminator("swapBaseOutput"));
        println!("deposit: {:?}", instruction_discriminator("deposit"));
        println!("initialize: {:?}", instruction_discriminator("initialize"));
        println!("withdraw: {:?}", instruction_discriminator("withdraw"));
    }

    #[test]
    fn test_raydium_events() {
        // Test Raydium CPMM events
        println!("SwapEvent: {:?}", event_discriminator("SwapEvent"));
        println!("LpChangeEvent: {:?}", event_discriminator("LpChangeEvent"));
    }
}
