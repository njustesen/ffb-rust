/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerSkill.
/// Abstract handler for `/skill` command — adds or removes skills from players.
use ffb_model::model::team::Team;
use ffb_model::factory::skill_factory::SkillFactory;
use ffb_model::enums::SkillId;
use super::talk_requirements::{Client, Environment, Privilege};

const ADD: &str = "add";
const REMOVE: &str = "remove";

pub struct TalkHandlerSkill {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
    skill_factory: SkillFactory,
}

impl TalkHandlerSkill {
    /// Java: `super("/skill", 3, ...)`.
    pub const COMMAND: &'static str = "/skill";
    pub const COMMAND_PARTS_THRESHOLD: usize = 3;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self {
            required_client,
            required_environment,
            requires_one_privilege_of,
            skill_factory: SkillFactory::new(),
        }
    }

    /// Java: `handle(...)` — `commands[1]` is `add`/`remove`, `commands[2]` the skill name
    /// (underscores replaced with spaces before lookup), and every player number from
    /// `commandPartsThreshold` onward (or `all`) is affected. Returns the info message for
    /// each affected player (Java sends one `sendPlayerTalk` per player).
    pub fn handle(&self, team: &mut Team, commands: &[String]) -> Vec<String> {
        if commands.len() <= Self::COMMAND_PARTS_THRESHOLD {
            return Vec::new();
        }
        let Some(skill) = self.skill_factory.for_name(&commands[2].replace('_', " ")) else {
            return Vec::new();
        };
        let action = commands[1].clone();
        let player_ids = find_player_ids_in_command(team, commands, Self::COMMAND_PARTS_THRESHOLD);

        let mut messages = Vec::new();
        for id in player_ids {
            let Some(player) = team.player_mut(&id) else { continue };
            if action == ADD {
                player.add_skill(skill);
                messages.push(format!("Added skill {} to player {}.", skill_name(skill), player.name));
            } else if action == REMOVE {
                player.remove_skill(skill);
                messages.push(format!("Removed skill {} from player {}.", skill_name(skill), player.name));
            }
        }
        messages
    }
}

/// Java: `Skill.getName()` — display name; the Rust `SkillId` only exposes a Java
/// class-name via `SkillFactory`, so this falls back to the class name for reporting.
fn skill_name(skill: SkillId) -> String {
    skill.class_name().to_string()
}

/// Java: `TalkHandler.findPlayersInCommand`. Duplicated locally (and in the other
/// `handler/talk/` files that need it) because the abstract `TalkHandler` base class is
/// owned by a concurrent translation pass.
pub(super) fn find_player_ids_in_command(team: &Team, commands: &[String], threshold: usize) -> Vec<String> {
    let mut ids = std::collections::HashSet::new();
    if commands.len() > threshold {
        if commands[threshold].eq_ignore_ascii_case("all") {
            ids.extend(team.players.iter().map(|p| p.id.clone()));
        } else {
            for token in &commands[threshold..] {
                if let Ok(nr) = token.parse::<i32>() {
                    if let Some(p) = team.player_by_nr(nr) {
                        ids.insert(p.id.clone());
                    }
                }
            }
        }
    }
    ids.into_iter().collect()
}

impl Default for TalkHandlerSkill {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::player::Player;

    fn team_with_players() -> Team {
        Team {
            id: "home".into(), name: "Home".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], vampire_lord: false, necromancer: false,
            players: vec![
                Player { id: "p1".into(), name: "Alice".into(), nr: 1, ..Player::default() },
                Player { id: "p2".into(), name: "Bob".into(), nr: 2, ..Player::default() },
            ],
        }
    }

    #[test]
    fn construct() {
        let h = TalkHandlerSkill::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_adds_skill_to_named_player() {
        let h = TalkHandlerSkill::default();
        let mut team = team_with_players();
        let commands = vec!["/skill".into(), ADD.into(), "Block".into(), "1".into()];
        let messages = h.handle(&mut team, &commands);
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("Added skill"));
        assert!(team.player("p1").unwrap().has_skill(SkillId::Block));
    }

    #[test]
    fn handle_removes_skill_from_all_players() {
        let h = TalkHandlerSkill::default();
        let mut team = team_with_players();
        team.player_mut("p1").unwrap().add_skill(SkillId::Dodge);
        team.player_mut("p2").unwrap().add_skill(SkillId::Dodge);
        let commands = vec!["/skill".into(), REMOVE.into(), "Dodge".into(), "all".into()];
        let messages = h.handle(&mut team, &commands);
        assert_eq!(messages.len(), 2);
        assert!(!team.player("p1").unwrap().has_skill(SkillId::Dodge));
        assert!(!team.player("p2").unwrap().has_skill(SkillId::Dodge));
    }

    #[test]
    fn handle_unknown_skill_returns_empty() {
        let h = TalkHandlerSkill::default();
        let mut team = team_with_players();
        let commands = vec!["/skill".into(), ADD.into(), "not_a_skill".into(), "1".into()];
        assert!(h.handle(&mut team, &commands).is_empty());
    }
}
