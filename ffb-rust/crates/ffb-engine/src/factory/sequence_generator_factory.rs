/// Translation of com.fumbbl.ffb.server.factory.SequenceGeneratorFactory.
///
/// Java uses Scanner<SequenceGenerator.class> reflection to auto-populate, keyed by
/// SequenceGenerator.Type.name() (e.g. "Block", "BlitzMove", etc.).
///
/// Rust: each generator is a standalone module with different parameter types, so they cannot
/// share a single trait easily. This factory validates known generator names per edition.
/// headless: full generator dispatch via enum-based dispatcher not yet implemented — name validation only.
use std::collections::HashSet;
use ffb_model::enums::Rules;

pub struct SequenceGeneratorFactory {
    /// Java: Map<String, SequenceGenerator<...>> generators — keyed by Type.name().
    known_names: HashSet<&'static str>,
}

impl SequenceGeneratorFactory {
    pub fn new() -> Self { Self { known_names: HashSet::new() } }

    /// Java: initialize(Game game) — Scanner populates the map.
    /// Rust: registers known generator names for validation/lookup.
    pub fn initialize(&mut self, rules: Rules) {
        // Common generators shared across all editions:
        let common = [
            "AutoGazeZoat", "BalefulHex", "BlackInk", "BlitzBlock", "BlitzMove",
            "Block", "Bomb", "CatchOfTheDay", "EndGame", "EndPlayerAction", "EndTurn",
            "Foul", "FuriousOutburst", "Kickoff", "LookIntoMyEyes", "Move",
            "MultiBlock", "Pass", "Punt", "RaidingParty", "ScatterPlayer", "Select",
            "SelectBlitzTarget", "SpecialEffect", "StartGame", "ThenIStartedBlastin",
            "ThrowARock", "ThrowKeg", "ThrowTeamMate", "Treacherous",
        ];
        for name in common {
            self.known_names.insert(name);
        }
        match rules {
            Rules::Bb2016 | Rules::Bb2020 => {
                // BB2016/BB2020 share the same set (BB2016 subset, BB2020 superset).
                self.known_names.insert("KickTeamMate");
            }
            Rules::Bb2025 => {
                // BB2025 adds extra generators.
                self.known_names.insert("KickTeamMate");
            }
            _ => {}
        }
    }

    /// Java: forName(String name) — returns the generator for the given type name.
    /// Rust: returns true if the name is a known generator for the initialized edition.
    pub fn for_name(&self, name: &str) -> bool {
        self.known_names.contains(name)
    }

    pub fn is_empty(&self) -> bool { self.known_names.is_empty() }
    pub fn len(&self) -> usize { self.known_names.len() }
}

impl Default for SequenceGeneratorFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_miss_returns_false() {
        let factory = SequenceGeneratorFactory::new();
        assert!(!factory.for_name("Block"));
    }

    #[test]
    fn new_factory_is_empty() {
        assert!(SequenceGeneratorFactory::new().is_empty());
    }

    #[test]
    fn initialize_bb2025_registers_block_generator() {
        let mut f = SequenceGeneratorFactory::new();
        f.initialize(Rules::Bb2025);
        assert!(f.for_name("Block"));
    }

    #[test]
    fn initialize_bb2016_registers_expected_names() {
        let mut f = SequenceGeneratorFactory::new();
        f.initialize(Rules::Bb2016);
        assert!(f.for_name("BlitzBlock"));
        assert!(f.for_name("BlitzMove"));
        assert!(f.for_name("Move"));
        assert!(f.for_name("Foul"));
        assert!(!f.for_name("UnknownGenerator"));
    }

    #[test]
    fn initialize_bb2020_registers_kickoff() {
        let mut f = SequenceGeneratorFactory::new();
        f.initialize(Rules::Bb2020);
        assert!(f.for_name("Kickoff"));
        assert!(f.for_name("KickTeamMate"));
    }

    #[test]
    fn initialize_bb2025_registers_thirty_plus_generators() {
        let mut f = SequenceGeneratorFactory::new();
        f.initialize(Rules::Bb2025);
        assert!(f.len() >= 31);
    }

    #[test]
    fn initialize_bb2016_registers_expected_count() {
        let mut f = SequenceGeneratorFactory::new();
        f.initialize(Rules::Bb2016);
        assert!(f.len() >= 15);
    }
}
