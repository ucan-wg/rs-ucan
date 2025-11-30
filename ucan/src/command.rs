//! Command helpers.

use serde::{Deserialize, Serialize, Serializer};

/// Command type representing a sequence of command segments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct Command(pub Vec<String>);

impl Command {
    /// Create a new Command from a vector of strings.
    #[must_use]
    pub const fn new(segments: Vec<String>) -> Self {
        Command(segments)
    }

    /// Get the segments of the command.
    #[must_use]
    pub const fn segments(&self) -> &Vec<String> {
        &self.0
    }

    /// Check if the command starts with the given prefix.
    #[must_use]
    pub fn starts_with(&self, prefix: &Command) -> bool {
        if prefix.0.len() > self.0.len() {
            return false;
        }
        self.0.iter().zip(&prefix.0).all(|(a, b)| a == b)
    }
}

impl From<Vec<String>> for Command {
    fn from(segments: Vec<String>) -> Self {
        Command::new(segments)
    }
}

impl From<Command> for Vec<String> {
    fn from(command: Command) -> Self {
        command.0
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cleaned = self
            .0
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        if cleaned.is_empty() {
            f.write_str("/")
        } else {
            write!(f, "/{}/", cleaned.join("/"))
        }
    }
}

impl Serialize for Command {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let trimmed = s.trim_matches('/');
        let parts: Vec<String> = trimmed
            .split('/')
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect();
        Ok(Command(parts))
    }
}
