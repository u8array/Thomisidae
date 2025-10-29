pub mod schema;
pub mod handler;
pub mod extractors;
pub mod content;
pub mod chunk;

pub use handler::FetchTextHandler;
pub use schema::meta;

pub use extractors::{extract_best_blocks, extract_fallback_blocks};
#[cfg(feature = "readability")]
pub use extractors::extract_readability;
