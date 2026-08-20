/// Translations of com.fumbbl.ffb.server.injury.modification package.
///
/// Java: abstract base class InjuryContextModification<T extends ModificationParams>
/// plus concrete implementations for each skill-based injury modifier.

pub mod injury_context_modification;
pub mod modification_params;
pub mod old_pro_modification_params;
pub mod av_or_inj_modification;
pub mod brutal_block_modification;
pub mod crushing_blow_modification;
pub mod ghostly_flames_modification;
pub mod master_assassin_modification;
pub mod old_pro_modification;
pub mod savage_mauling_modification;
pub mod bb2020;
pub mod bb2025;

pub use injury_context_modification::InjuryContextModification;
pub use modification_params::ModificationParams;
pub use old_pro_modification_params::OldProModificationParams;

use ffb_model::enums::{Rules, SkillId};
use ffb_model::model::game::Game;

/// Java: each SkillBehaviour registers its InjuryContextModification via
/// `registerModifier(new XxxModification())` — this is the Rust equivalent of that registry,
/// keyed by SkillId (edition-specific behaviours resolve to edition-specific modifications).
pub(crate) fn modification_for_skill(skill: SkillId, rules: Rules) -> Option<Box<dyn InjuryContextModification>> {
    match skill {
        // mixed/OldProBehaviour (BB2020 + BB2025)
        SkillId::OldPro => Some(Box::new(old_pro_modification::OldProModification::new())),
        // mixed/CrushingBlowBehaviour
        SkillId::CrushingBlow => Some(Box::new(crushing_blow_modification::CrushingBlowModification::new())),
        // mixed/SavageMaulingBehaviour
        SkillId::SavageMauling => Some(Box::new(savage_mauling_modification::SavageMaulingModification::new())),
        // mixed/RamBehaviour, bb2020/DwarfenScourgeBehaviour, bb2025/DwarvenScourgeBehaviour
        SkillId::Ram | SkillId::DwarfenScourge | SkillId::DwarvenScourge =>
            Some(Box::new(av_or_inj_modification::AvOrInjModification::new())),
        // bb2020/BrutalBlockBehaviour
        SkillId::BrutalBlock => Some(Box::new(brutal_block_modification::BrutalBlockModification::new())),
        // bb2020/GhostlyFlamesBehaviour
        SkillId::GhostlyFlames => Some(Box::new(ghostly_flames_modification::GhostlyFlamesModification::new())),
        // bb2020/ vs bb2025/MasterAssassinBehaviour
        SkillId::MasterAssassin => Some(if rules == Rules::Bb2025 {
            Box::new(bb2025::master_assassin_modification::MasterAssassinModification::new())
        } else {
            Box::new(master_assassin_modification::MasterAssassinModification::new())
        }),
        // bb2020/ vs bb2025/SlayerBehaviour
        SkillId::Slayer => Some(if rules == Rules::Bb2025 {
            Box::new(bb2025::slayer_modification::SlayerModification::new())
        } else {
            Box::new(bb2020::slayer_modification::SlayerModification::new())
        }),
        // bb2020/ vs bb2025/ToxinConnoisseurBehaviour
        SkillId::ToxinConnoisseur => Some(if rules == Rules::Bb2025 {
            Box::new(bb2025::toxin_connoisseur_modification::ToxinConnoisseurModification::new())
        } else {
            Box::new(bb2020::toxin_connoisseur_modification::ToxinConnoisseurModification::new())
        }),
        // bb2025/KrumpAndSmashBehaviour, bb2025/LoneFoulerBehaviour
        SkillId::KrumpAndSmash => Some(Box::new(bb2025::krump_and_smash_modification::KrumpAndSmashModification::new())),
        SkillId::LoneFouler => Some(Box::new(bb2025::lone_fouler_modification::LoneFoulerModification::new())),
        _ => None,
    }
}

/// Java: `Player.getUnusedInjuryModification(InjuryType)` — the first UNUSED skill whose
/// behaviour carries an InjuryContextModification valid for this injury type.
pub fn unused_injury_modification(
    game: &Game,
    player_id: &str,
    injury_type_name: &str,
) -> Option<Box<dyn InjuryContextModification>> {
    let player = game.player(player_id)?;
    for skill in player.all_skill_ids() {
        if player.used_skills.contains(&skill) {
            continue;
        }
        if let Some(mut m) = modification_for_skill(skill, game.rules) {
            if m.is_valid_type(injury_type_name) {
                m.set_skill_id(skill);
                return Some(m);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::model::player::Player;

    fn game_with_skilled_player(skill: SkillId) -> Game {
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, position_id: "lineman".into(),
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![ffb_model::model::skill_def::SkillWithValue { skill_id: skill, value: None }],
            ..Default::default()
        });
        game
    }

    #[test]
    fn unused_injury_modification_finds_old_pro_for_chainsaw() {
        // Java Player.getUnusedInjuryModification: Helmut Wulf's chainsaw kickback
        // (attacker == null → DEFENDER lookup) must find his unused Old Pro.
        let game = game_with_skilled_player(SkillId::OldPro);
        let m = unused_injury_modification(&game, "h1", "Chainsaw");
        assert!(m.is_some(), "unused Old Pro must be found for a Chainsaw injury");
        assert!(m.unwrap().requires_conditional_re_roll_skill());
    }

    #[test]
    fn used_skill_yields_no_modification() {
        let mut game = game_with_skilled_player(SkillId::OldPro);
        game.team_home.players[0].used_skills.insert(SkillId::OldPro);
        assert!(unused_injury_modification(&game, "h1", "Chainsaw").is_none());
    }

    #[test]
    fn invalid_type_yields_no_modification() {
        // Old Pro's validTypes has no "CrowdPush".
        let game = game_with_skilled_player(SkillId::OldPro);
        assert!(unused_injury_modification(&game, "h1", "CrowdPush").is_none());
    }
}
