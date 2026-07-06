use serde::{Deserialize, Serialize};
use crate::model::player::Player;
use crate::model::zapped_position::ZappedPosition;

/// Java: ZappedPlayer — wraps the original RosterPlayer and exposes ZappedPosition stats.
/// Created when a player is successfully ZAP-ped by a card effect.
/// The original player is stored and restored at end of drive/half.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZappedPlayer {
    /// Java: fOriginalPlayer — the real player before zap.
    pub original_player: Player,
    /// Java: fPosition — the position with frog stats and skills.
    pub position: ZappedPosition,
}

impl ZappedPlayer {
    /// Java: ZappedPlayer.init(pPlayer, game).
    pub fn new(original_player: Player, position: ZappedPosition) -> Self {
        Self { original_player, position }
    }

    /// Java: ZappedPlayer.getId() delegates to originalPlayer.
    pub fn get_id(&self) -> &str { &self.original_player.id }

    /// Java: ZappedPlayer.getOriginalPlayer().
    pub fn get_original_player(&self) -> &Player { &self.original_player }

    /// Java: ZappedPlayer.getName() delegates to originalPlayer.
    pub fn get_name(&self) -> &str { &self.original_player.name }

    /// Java: ZappedPlayer.getNr() delegates to originalPlayer.
    pub fn get_nr(&self) -> i32 { self.original_player.nr }

    /// Java: ZappedPlayer.getAgility() returns position.getAgility().
    pub fn get_agility(&self) -> i32 { self.position.get_agility() }

    /// Java: ZappedPlayer.getPassing() returns position.getPassing().
    pub fn get_passing(&self) -> i32 { self.position.get_passing() }

    /// Java: ZappedPlayer.getArmour() returns position.getArmour().
    pub fn get_armour(&self) -> i32 { self.position.get_armour() }

    /// Java: ZappedPlayer.getMovement() returns position.getMovement().
    pub fn get_movement(&self) -> i32 { self.position.get_movement() }

    /// Java: ZappedPlayer.getStrength() returns position.getStrength().
    pub fn get_strength(&self) -> i32 { self.position.get_strength() }

    /// Java: ZappedPlayer.getSkills() returns position.getSkills().
    pub fn get_skills(&self) -> Vec<crate::model::skill_def::SkillWithValue> {
        ZappedPosition::get_skills()
    }

    /// Java: ZappedPlayer.getLastingInjuries() delegates to originalPlayer.
    pub fn get_lasting_injuries(&self) -> &[crate::enums::SeriousInjuryKind] {
        &self.original_player.stat_injuries
    }

    /// Java: ZappedPlayer.getUrlPortrait() delegates to originalPlayer.
    pub fn get_url_portrait(&self) -> Option<&str> { None }

    /// Java: ZappedPlayer.getCurrentSpps() delegates to originalPlayer.
    pub fn get_current_spps(&self) -> i32 { self.original_player.current_spps }

    /// Java: ZappedPlayer.getPositionId() delegates to originalPlayer.
    pub fn get_position_id(&self) -> &str { &self.original_player.position_id }

    /// Java: ZappedPlayer.getRace() delegates to originalPlayer.
    pub fn get_race(&self) -> Option<&str> { self.original_player.race.as_deref() }

    /// Java: ZappedPlayer.isJourneyman() delegates to originalPlayer.
    pub fn is_journeyman(&self) -> bool { self.original_player.is_journeyman() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerType, PlayerGender};

    fn player_for_test() -> Player {
        Player {
            id: "p42".into(),
            name: "Bob Blockyhead".into(),
            nr: 7,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            ..Default::default()
        }
    }

    fn zap_bb2020(player: Player) -> ZappedPlayer {
        let pos = ZappedPosition::new_bb2020(
            player.position_id.clone(),
            player.name.clone(),
        );
        ZappedPlayer::new(player, pos)
    }

    #[test]
    fn get_id_delegates_to_original() {
        let zapped = zap_bb2020(player_for_test());
        assert_eq!(zapped.get_id(), "p42");
    }

    #[test]
    fn get_name_delegates_to_original() {
        let zapped = zap_bb2020(player_for_test());
        assert_eq!(zapped.get_name(), "Bob Blockyhead");
    }

    #[test]
    fn get_nr_delegates_to_original() {
        let zapped = zap_bb2020(player_for_test());
        assert_eq!(zapped.get_nr(), 7);
    }

    #[test]
    fn stats_come_from_zapped_position_not_original() {
        let zapped = zap_bb2020(player_for_test());
        assert_eq!(zapped.get_movement(), 5);
        assert_eq!(zapped.get_strength(), 1);
        assert_eq!(zapped.get_agility(), 2);
        assert_eq!(zapped.get_passing(), 0);
        assert_eq!(zapped.get_armour(), 5);
    }

    #[test]
    fn bb2016_zap_has_different_agility() {
        let player = player_for_test();
        let pos = ZappedPosition::new_bb2016(player.position_id.clone(), player.name.clone());
        let zapped = ZappedPlayer::new(player, pos);
        assert_eq!(zapped.get_agility(), 4);
    }

    #[test]
    fn get_skills_returns_six_zap_skills() {
        let zapped = zap_bb2020(player_for_test());
        assert_eq!(zapped.get_skills().len(), 6);
    }

    #[test]
    fn get_original_player_is_unmodified() {
        let zapped = zap_bb2020(player_for_test());
        let orig = zapped.get_original_player();
        assert_eq!(orig.movement, 6);
        assert_eq!(orig.strength, 3);
    }

    #[test]
    fn serde_round_trip() {
        let zapped = zap_bb2020(player_for_test());
        let json = serde_json::to_string(&zapped).unwrap();
        let back: ZappedPlayer = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), zapped.get_id());
        assert_eq!(back.get_agility(), zapped.get_agility());
    }
}
