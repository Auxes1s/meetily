pub mod commands;
pub mod manager;
pub mod models;
pub mod repositories;
pub mod setup;

pub mod notes_commands;
pub mod deletion;
pub mod library;
pub mod workspace_commands;
pub mod export;
pub mod recording_sessions;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod workspace_contract_tests;
