/// Translation of com.fumbbl.ffb.server.injury.modification.SavageMaulingModification.
///
/// Valid for many injury types (Block, BlockStunned, BlockProne, Foul, FoulForSpp,
/// ProjectileVomit, Stab). Re-rolls the injury. allowed = always true (same-team ok).
/// Gate: not casualty, OR spotted foul (injury dice equal + foul type), OR
/// (same team + not AnimalSavagery + not stunned + has tacklezones).
use ffb_model::enums::ApothecaryMode;
use ffb_model::model::game::Game;
use ffb_model::model::SkillUse;
use ffb_model::util::rng::GameRng;
use crate::injury::InjuryContext;
use crate::injury::modification::{InjuryContextModification, ModificationParams};

pub struct SavageMaulingModification {
    skill_id: Option<u16>,
}

impl SavageMaulingModification {
    pub fn new() -> Self { Self { skill_id: None } }

    fn is_spotted_foul(ctx: &InjuryContext, injury_type_name: &str) -> bool {
        if let Some([d1, d2]) = ctx.injury_roll {
            let is_foul = matches!(injury_type_name, "Foul" | "FoulForSpp" | "FoulWithChainsaw" | "FoulForSppWithChainsaw");
            d1 == d2 && is_foul
        } else {
            false
        }
    }
}

impl Default for SavageMaulingModification {
    fn default() -> Self { Self::new() }
}

const VALID: &[&str] = &["Block", "BlockStunned", "BlockProne", "Foul", "FoulForSpp", "ProjectileVomit", "Stab"];

impl InjuryContextModification for SavageMaulingModification {
    fn skill_use(&self) -> SkillUse { SkillUse::RE_ROLL_INJURY }
    fn valid_types(&self) -> &'static [&'static str] { VALID }
    fn skill_id(&self) -> Option<u16> { self.skill_id }
    fn set_skill_id(&mut self, id: u16) { self.skill_id = Some(id); }

    fn allowed_for_attacker_and_defender_teams(&self, _game: &Game, _ctx: &InjuryContext) -> bool {
        true
    }

    fn try_injury_modification(&self, game: &Game, ctx: &InjuryContext, injury_type_name: &str) -> bool {
        if !ctx.is_casualty() {
            return true;
        }
        if Self::is_spotted_foul(ctx, injury_type_name) {
            return true;
        }
        // same team + not animal savagery + not stunned + has tacklezones
        let same_team = !self.different_teams(game, ctx);
        let not_savagery = ctx.apothecary_mode != ApothecaryMode::AnimalSavagery;
        let not_stunned = ctx.injury.map(|s| !s.is_stunned()).unwrap_or(true);
        same_team && not_savagery && not_stunned && self.acting_player_has_tacklezones(game)
    }

    fn modify_injury_internal(&self, game: &Game, rng: &mut GameRng, ctx: &mut InjuryContext) -> bool {
        let d1 = rng.d6();
        let d2 = rng.d6();
        ctx.set_injury_roll([d1, d2]);
        ctx.injury = self.interpret_injury(game, ctx);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STUNNED, PS_BADLY_HURT, PS_SERIOUS_INJURY, PlayerState};
    use ffb_model::model::player::Player;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use crate::step::framework::test_team;

    fn make() -> SavageMaulingModification { SavageMaulingModification::new() }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 7,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None, ..Default::default()
        };
        if home { game.team_home.players.push(p); }
        else { game.team_away.players.push(p); }
    }

    #[test]
    fn valid_types() {
        let m = make();
        assert!(m.is_valid_type("Block"));
        assert!(m.is_valid_type("Foul"));
        assert!(m.is_valid_type("Stab"));
        assert!(!m.is_valid_type("Chainsaw"));
    }

    #[test]
    fn allows_same_team() {
        let game = make_game();
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(make().allowed_for_attacker_and_defender_teams(&game, &ctx));
    }

    #[test]
    fn try_injury_true_when_not_casualty() {
        let game = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_STUNNED));
        assert!(make().try_injury_modification(&game, &ctx, "Block"));
    }

    #[test]
    fn try_injury_true_for_casualty_with_spotted_foul() {
        let game = make_game();
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_BADLY_HURT));
        ctx.injury_roll = Some([3, 3]); // doubles
        assert!(make().try_injury_modification(&game, &ctx, "Foul"));
    }

    #[test]
    fn try_injury_false_for_casualty_different_teams_no_spotted_foul() {
        let mut game = make_game();
        add_player(&mut game, true, "att");
        add_player(&mut game, false, "def");
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury = Some(PlayerState::new(PS_SERIOUS_INJURY));
        ctx.injury_roll = Some([3, 4]);
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        assert!(!make().try_injury_modification(&game, &ctx, "Block"));
    }

    #[test]
    fn try_injury_false_for_casualty_animal_savagery() {
        let mut game = make_game();
        add_player(&mut game, true, "att");
        add_player(&mut game, true, "def"); // same team
        let mut ctx = InjuryContext::new(ApothecaryMode::AnimalSavagery);
        ctx.injury = Some(PlayerState::new(PS_BADLY_HURT));
        ctx.attacker_id = Some("att".into());
        ctx.defender_id = Some("def".into());
        assert!(!make().try_injury_modification(&game, &ctx, "Block"));
    }

    #[test]
    fn modify_injury_internal_sets_injury_roll_and_injury() {
        let game = make_game();
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        let result = make().modify_injury_internal(&game, &mut rng, &mut ctx);
        assert!(result);
        assert!(ctx.injury_roll.is_some());
        let [d1, d2] = ctx.injury_roll.unwrap();
        assert!((1..=6).contains(&d1));
        assert!((1..=6).contains(&d2));
    }

    #[test]
    fn is_spotted_foul_false_for_non_foul_type() {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.injury_roll = Some([4, 4]);
        assert!(!SavageMaulingModification::is_spotted_foul(&ctx, "Block"));
    }

    #[test]
    fn is_spotted_foul_false_when_no_injury_roll() {
        let ctx = InjuryContext::new(ApothecaryMode::Defender);
        assert!(!SavageMaulingModification::is_spotted_foul(&ctx, "Foul"));
    }
}
