/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerRoll.
/// Handles `/roll` command — queues or clears test dice rolls.
///
/// Java keys queued rolls by `DiceCategory` (per-team, per-die-type) and reports the
/// queued rolls back to the coach. The Rust `ffb_engine::dice_roller::DiceRoller` (owned
/// outside `handler/talk/`) only exposes a flat `category -> Vec<i32>` map with no getter,
/// so this translation covers the add/clear side faithfully and reports a simplified
/// confirmation message instead of echoing the exact per-category queue contents.
use ffb_engine::dice_roller::DiceRoller;
use super::talk_requirements::{Client, Environment};

pub struct TalkHandlerRoll {
    pub required_client: Client,
    pub required_environment: Environment,
}

impl TalkHandlerRoll {
    pub const COMMAND: &'static str = "/roll";
    pub const COMMAND_PARTS_THRESHOLD: usize = 0;

    /// Java: `super("/roll", 0, Client.PLAYER, Environment.TEST_GAME)`.
    pub fn new() -> Self {
        Self { required_client: Client::Player, required_environment: Environment::TestGame }
    }

    /// Java: `handle(...)` — `clear` empties the test-roll queue, otherwise every trailing
    /// token that parses as an integer is queued as a "General" category test roll.
    pub fn handle(&self, roller: &mut DiceRoller, commands: &[String]) -> String {
        if commands.len() > 1 {
            if commands[1] == "clear" {
                roller.clear_test_rolls();
            } else {
                for token in &commands[1..] {
                    if let Ok(roll) = token.parse::<i32>() {
                        roller.add_test_roll("General", roll);
                    }
                }
            }
        }
        "Next dice rolls will be as configured.".to_string()
    }
}

impl Default for TalkHandlerRoll {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let h = TalkHandlerRoll::new();
        assert_eq!(h.required_client, Client::Player);
        assert_eq!(h.required_environment, Environment::TestGame);
    }

    #[test]
    fn handle_queues_numeric_tokens_as_test_rolls() {
        let h = TalkHandlerRoll::new();
        let mut roller = DiceRoller::new();
        let commands = vec!["/roll".to_string(), "3".to_string(), "5".to_string()];
        h.handle(&mut roller, &commands);
        assert_eq!(roller.roll_dice(6), 3);
        assert_eq!(roller.roll_dice(6), 5);
    }

    #[test]
    fn handle_clear_empties_queue() {
        let h = TalkHandlerRoll::new();
        let mut roller = DiceRoller::new();
        roller.add_test_roll("General", 4);
        let commands = vec!["/roll".to_string(), "clear".to_string()];
        h.handle(&mut roller, &commands);
        // With the queue cleared, roll_dice falls through to the real RNG path.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| roller.roll_dice(6)));
        assert!(result.is_err(), "expected fallthrough to the not-yet-wired Fortuna PRNG");
    }

    #[test]
    fn handle_with_no_args_is_noop() {
        let h = TalkHandlerRoll::new();
        let mut roller = DiceRoller::new();
        let commands = vec!["/roll".to_string()];
        let info = h.handle(&mut roller, &commands);
        assert!(info.contains("random") || info.contains("configured"));
    }
}
