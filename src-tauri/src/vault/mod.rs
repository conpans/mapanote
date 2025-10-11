mod frontmatter;
mod notes;
mod reader;
mod writer; // ← NEW

pub use reader::VaultReader;
pub use writer::VaultWriter; // ← NEW
