/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStat.
/// Abstract handler for `/stat` command — modifies player stat values (MA, ST, AG, AV, PA).
use ffb_model::model::team::Team;
use super::talk_handler_skill::find_player_ids_in_command;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerStat {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerStat {
    /// Java: `super("/stat", 3, ...)`.
    pub const COMMAND: &'static str = "/stat";
    pub const COMMAND_PARTS_THRESHOLD: usize = 3;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self { required_client, required_environment, requires_one_privilege_of }
    }

    /// Java: `handle(...)` — `commands[1]` is the stat code (`ma`/`st`/`ag`/`pa`/`av`,
    /// case-insensitive), `commands[2]` the new non-negative value, and every player named
    /// from `commandPartsThreshold` onward (or `all`) is updated.
    pub fn handle(&self, team: &mut Team, commands: &[String]) -> Vec<String> {
        let Some(stat_token) = commands.get(2) else { return Vec::new() };
        let Ok(stat) = stat_token.parse::<i32>() else { return Vec::new() };
        if stat < 0 {
            return Vec::new();
        }
        let Some(stat_name) = commands.get(1) else { return Vec::new() };
        let stat_name = stat_name.to_lowercase();

        let mut messages = Vec::new();
        for id in find_player_ids_in_command(team, commands, Self::COMMAND_PARTS_THRESHOLD) {
            let Some(player) = team.player_mut(&id) else { continue };
            let (label, applied) = match stat_name.as_str() {
                "ma" => { player.movement = stat; ("MA", true) }
                "st" => { player.strength = stat; ("ST", true) }
                "ag" => { player.agility = stat; ("AG", true) }
                "pa" => { player.passing = stat; ("PA", true) }
                "av" => { player.armour = stat; ("AV", true) }
                _ => ("", false),
            };
            if applied {
                messages.push(format!("Set {label} stat of player {} to {stat}.", player.name));
            }
        }
        messages
    }
}

impl Default for TalkHandlerStat {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::player::Player;

    fn team() -> Team {
        Team {
            id: "home".into(), name: "Home".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], vampire_lord: false, necromancer: false,
            players: vec![Player { id: "p1".into(), name: "Alice".into(), nr: 1, movement: 6, ..Player::default() }],
        }
    }

    #[test]
    fn construct() {
        let h = TalkHandlerStat::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_sets_movement_stat() {
        let h = TalkHandlerStat::default();
        let mut team = team();
        let commands = vec!["/stat".into(), "ma".into(), "8".into(), "1".into()];
        let messages = h.handle(&mut team, &commands);
        assert_eq!(messages, vec!["Set MA stat of player Alice to 8.".to_string()]);
        assert_eq!(team.player("p1").unwrap().movement, 8);
    }

    #[test]
    fn handle_rejects_negative_stat() {
        let h = TalkHandlerStat::default();
        let mut team = team();
        let commands = vec!["/stat".into(), "ma".into(), "-1".into(), "1".into()];
        assert!(h.handle(&mut team, &commands).is_empty());
        assert_eq!(team.player("p1").unwrap().movement, 6);
    }

    #[test]
    fn handle_unknown_stat_code_is_noop() {
        let h = TalkHandlerStat::default();
        let mut team = team();
        let commands = vec!["/stat".into(), "xx".into(), "8".into(), "1".into()];
        assert!(h.handle(&mut team, &commands).is_empty());
    }
}
