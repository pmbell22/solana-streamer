pub mod idl;
pub mod parser;
pub mod types;
pub mod streaming;

pub use idl::{Idl, DexProtocol};
pub use parser::InstructionParser;
pub use types::{DexEvent, ParsedInstruction, ParsedInstructionData};
pub use streaming::DexStreamParser;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::idl::{load_idl_from_file, load_idl_from_json, DexProtocol, Idl};
    pub use crate::parser::InstructionParser;
    pub use crate::streaming::DexStreamParser;
    pub use crate::types::{DexEvent, ParsedInstruction, ParsedInstructionData};
}
