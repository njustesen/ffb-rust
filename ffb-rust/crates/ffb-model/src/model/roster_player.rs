/// 1:1 translation of `com.fumbbl.ffb.model.RosterPlayer`.
///
/// In Java, RosterPlayer extends Player<RosterPosition> — it is the concrete
/// implementation of the Player abstract class. In the Rust model, Player is
/// already the concrete type (stored in Team::players). RosterPlayer is therefore
/// a type alias for Player.
///
/// The following Java fields map to fields already on Player:
/// - fId → Player::id
/// - fNr → Player::nr
/// - fName → Player::name
/// - fPlayerType → Player::player_type
/// - playerStatus → Player::player_status (added in Phase Z)
/// - fPlayerGender → Player::gender
/// - fMovement/Strength/Agility/Passing/Armour → Player stats
/// - fPositionId → Player::position_id
/// - fSkills (extra earned skills) → Player::extra_skills
/// - fLastingInjuries → Player::stat_injuries
/// - fRecoveringInjury → Player::recovering_injury
/// - fCurrentSpps → Player::current_spps
/// - usedSkills → Player::used_skills
/// - temporaryModifiers → Player::temporary_stat_mods
/// - temporarySkills → Player::temporary_skills / temporary_skill_sources
///
/// Java-only fields not translated (client-only or out of scope):
/// - fUrlPortrait, fUrlIconSet, fNrOfIcons, fIconSetIndex — client-side display
/// - fPosition (RosterPosition object ref) — use position_id + roster lookup
/// - skillValues, displayValues — client-side display metadata
/// - temporaryProperties (ISkillProperty) — not yet ported
///
/// Methods are on Player directly (same impl block).
use crate::model::player::Player;

pub type RosterPlayer = Player;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::player_status::PlayerStatus;
    use crate::enums::{PlayerType, PlayerGender};

    fn make_roster_player() -> RosterPlayer {
        RosterPlayer {
            id: "test_p".into(),
            name: "Test Player".into(),
            nr: 5,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            player_status: PlayerStatus::ACTIVE,
            ..RosterPlayer::default()
        }
    }

    #[test]
    fn roster_player_new_is_active() {
        let p = RosterPlayer::default();
        assert_eq!(p.player_status, PlayerStatus::ACTIVE);
        assert!(!p.is_journeyman());
    }

    #[test]
    fn set_journeyman_status() {
        let mut p = make_roster_player();
        p.set_player_status(PlayerStatus::JOURNEYMAN);
        assert!(p.is_journeyman());
    }

    #[test]
    fn add_and_remove_skill() {
        use crate::model::skill_def::SkillId;
        let mut p = make_roster_player();
        p.add_skill(SkillId::Loner);
        assert!(p.has_skill(SkillId::Loner));
        p.remove_skill(SkillId::Loner);
        assert!(!p.has_skill(SkillId::Loner));
    }

    #[test]
    fn get_player_status_roundtrip() {
        let mut p = make_roster_player();
        assert_eq!(p.get_player_status(), PlayerStatus::ACTIVE);
        p.set_player_status(PlayerStatus::JOURNEYMAN);
        assert_eq!(p.get_player_status(), PlayerStatus::JOURNEYMAN);
    }

    #[test]
    fn roster_player_is_player() {
        // RosterPlayer IS Player — same type, same fields
        let p: RosterPlayer = RosterPlayer {
            id: "star1".into(),
            name: "Star Player".into(),
            nr: 1,
            player_type: PlayerType::Star,
            ..RosterPlayer::default()
        };
        assert_eq!(p.player_type, PlayerType::Star);
    }
}
