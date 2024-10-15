pub mod commands;
#[cfg(test)]
mod tests;

/// The prefix used when creation a farm with an explicitly provided ID
pub const EXPLICIT_FARM_ID_PREFIX: &str = "m-";

/// The prefix used when creation a farm with an auto-generated ID
pub const AUTO_FARM_ID_PREFIX: &str = "f-";
