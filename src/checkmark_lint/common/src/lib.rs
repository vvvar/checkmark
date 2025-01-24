mod rule;
mod violation;

// Convenience re-exports.
pub use common::{Config, MarkDownFile};
pub use markdown::mdast::*;
pub use markdown::unist::*;
pub use rule::*;
pub use violation::*;
