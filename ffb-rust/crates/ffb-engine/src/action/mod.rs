use serde::{Deserialize, Serialize};
use ffb_model::model::player::PlayerId;
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::Weather;
use ffb_mechanics::skills::SkillId;

/// Every decision an agent can make in response to a game prompt.
///
/// The engine's `apply()` method accepts one of these per active side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    // ── Pre-game ────────────────────────────────────────────────────────────
    /// Choose heads (true) or tails (false) for the coin flip.
    CoinChoice { heads: bool },
    /// Choose to receive (true) or kick (false).
    ReceiveChoice { receive: bool },
    /// Place a player on the field during setup.
    PlacePlayer { player_id: PlayerId, coord: FieldCoordinate },
    /// Signal that the team setup is complete.
    ConfirmSetup,

    // ── Kickoff ─────────────────────────────────────────────────────────────
    /// Kick the ball to this coordinate.
    KickBall { coord: FieldCoordinate },
    /// Declare which player receives the touchback.
    Touchback { player_id: PlayerId },

    // ── Activation ──────────────────────────────────────────────────────────
    /// Activate a player with the given action type.
    ActivatePlayer { player_id: PlayerId, player_action: PlayerActionChoice },
    /// End the current team's turn.
    EndTurn,

    // ── Movement ────────────────────────────────────────────────────────────
    /// Move the active player along the given path.
    Move { path: Vec<FieldCoordinate> },

    // ── Block ────────────────────────────────────────────────────────────────
    /// Declare a block against a target.
    Block { defender_id: PlayerId },
    /// Stab an adjacent opponent (no block dice; direct armor/injury roll).
    Stab { defender_id: PlayerId },
    /// Declare Multiple Block against two adjacent targets (BB2020+, requires MultipleBlock skill).
    MultiBlock { defender1_id: PlayerId, defender2_id: PlayerId },
    /// Choose which block die result to apply.
    BlockChoice { die_index: usize },
    /// Choose where to push the defender.
    PushTo { coord: FieldCoordinate },
    /// Declare whether the attacker follows up.
    FollowUp { follow_up: bool },
    /// Move to one adjacent safe square after a block (HitAndRun skill). None = decline.
    HitAndRun { coord: Option<FieldCoordinate> },
    /// Move to one adjacent square before a block resolves (Trickster skill).
    TricksterMove { coord: FieldCoordinate },

    // ── Pass ─────────────────────────────────────────────────────────────────
    /// Throw the ball (or team-mate) to this coordinate.
    Pass { coord: FieldCoordinate },
    /// Attempt to intercept the pass (declaring intent before the roll).
    Intercept { attempt: bool },

    // ── Hand-off ─────────────────────────────────────────────────────────────
    HandOff { receiver_id: PlayerId },

    // ── Foul ─────────────────────────────────────────────────────────────────
    Foul { target_id: PlayerId },

    // ── Special actions ───────────────────────────────────────────────────────
    /// Throw a team-mate to this coordinate.
    ThrowTeamMate { player_id: PlayerId, coord: FieldCoordinate },
    /// Kick a team-mate to this coordinate.
    KickTeamMate { player_id: PlayerId, coord: FieldCoordinate },
    /// Attempt Hypnotic Gaze on the target.
    HypnoticGaze { target_id: PlayerId },
    /// Use Breathe Fire against an adjacent opponent (BB2020+).
    BreatheFire { target_id: PlayerId },
    /// Use Projectile Vomit against an adjacent opponent (BB2020+, PutridRegurgitation).
    ProjectileVomit { target_id: PlayerId },
    /// Use Bombardier to throw a bomb.
    ThrowBomb { coord: FieldCoordinate },
    /// Punt the ball to a target coordinate (BB2025, Punt skill, half-action).
    Punt { coord: FieldCoordinate },
    /// Declare use of a wizard spell.
    WizardSpell { spell: WizardSpellChoice, coord: FieldCoordinate },

    // ── Skill usage ───────────────────────────────────────────────────────────
    /// Declare use / non-use of a skill when the engine prompts for it.
    UseSkill { skill_id: SkillId, use_skill: bool },
    /// Declare use / non-use of a re-roll.
    UseReRoll { use_reroll: bool },
    /// Use (or decline) the apothecary on an injured player.
    UseApothecary { player_id: PlayerId, use_apothecary: bool },
    /// Use a bribe to avoid ejection.
    UseBribe { use_bribe: bool },
    /// Choose whether to argue the call after a foul ejection.
    ArgueTheCall { argue: bool },

    // ── Inducement ────────────────────────────────────────────────────────────
    /// Purchase inducements before the game.
    BuyInducements { purchases: Vec<InducementPurchase> },
    /// Play a card from the hand.
    PlayCard { card_id: String },

    // ── Star-player special attacks ───────────────────────────────────────
    /// PrimalSavagery: lash out against an adjacent opponent (D6+ST vs D6+AV).
    LashOut { target_id: PlayerId },
    /// TastyMorsel: bite an adjacent opponent (similar to Stab with custom modifiers).
    Bite { target_id: PlayerId },
    /// FuriousOutburst / TheFlashingBlade: armor roll against adjacent opponent instead of a block.
    ArmourRollAttack { target_id: PlayerId },
    /// BeerBarrelBash: throw a keg at a target coordinate (once-per-drive).
    ThrowKeg { coord: FieldCoordinate },
    /// CatchOfTheDay: attempt to grab the ball off the ground at activation (D6 >= 3).
    CatchOfTheDay,

    // ── Misc ─────────────────────────────────────────────────────────────────
    /// Select a player from a list prompt.
    SelectPlayer { player_id: PlayerId },
    /// Select a skill when levelling up.
    SelectSkill { skill_id: SkillId },
    /// Change the weather (Weather Mage).
    SelectWeather { weather: Weather },
    /// Acknowledge an information-only dialog.
    Acknowledge,
}

/// Which action type the agent wants to perform when activating a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerActionChoice {
    Move,
    Blitz,
    Block,
    Stab,
    Foul,
    Pass,
    HandOff,
    StandUp,
    StandUpBlitz,
    ThrowTeamMate,
    KickTeamMate,
    HypnoticGaze,
    ThrowBomb,
    Swoop,
    Punt,
    BreatheFire,
    ProjectileVomit,
    SecureTheBall,
}

/// Wizard spell choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WizardSpellChoice {
    Lightning,
    Fireball,
}

/// A single inducement purchase (id + quantity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InducementPurchase {
    pub id: String,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_coin_choice_serializes() {
        let a = Action::CoinChoice { heads: true };
        let json = serde_json::to_string(&a).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn action_end_turn_round_trips() {
        let a = Action::EndTurn;
        let json = serde_json::to_string(&a).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }
}
