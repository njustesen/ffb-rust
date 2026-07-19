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
    /// Activate a player with the given action type. `block_defender_id` is set for Block/Blitz.
    ActivatePlayer { player_id: PlayerId, player_action: PlayerActionChoice, block_defender_id: Option<PlayerId> },
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
    /// `target_id` is `Some` in multi-block sequences (CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET).
    BlockChoice { die_index: usize, target_id: Option<String> },
    /// Use Brawler skill to re-roll a BothDown result (CLIENT_USE_BRAWLER).
    /// `target_id` is `Some` in multi-block sequences.
    UseBrawler { target_id: Option<String> },
    /// Use Hatred skill to re-roll a Skull result (CLIENT_USE_HATRED).
    UseHatred,
    /// Use Pro skill to re-roll a single block die (CLIENT_USE_PRO_RE_ROLL_FOR_BLOCK).
    UseProReRollForBlock { die_index: usize },
    /// Use Consummate Professional skill to re-roll a single block die (CLIENT_USE_CONSUMMATE_RE_ROLL_FOR_BLOCK).
    UseConsummateReRollForBlock { die_index: usize },
    /// Use a single-die block re-roll skill (CLIENT_USE_SINGLE_BLOCK_DIE_RE_ROLL).
    UseSingleBlockDieReRoll { re_roll_source: String, die_index: usize },
    /// Use a multi-die block re-roll (Savage Blow, CLIENT_USE_MULTI_BLOCK_DICE_RE_ROLL).
    UseMultiBlockDiceReRoll { dice_indexes: Vec<usize> },
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
    /// Play a card from the hand, optionally targeting a player.
    PlayCard { card_id: String, target_player_id: Option<PlayerId> },
    /// Use a purchased inducement during the game (Java: CLIENT_USE_INDUCEMENT).
    /// `inducement_type` is the InducementType name (e.g. "WIZARD"); `card_id` is set for cards.
    UseInducement {
        inducement_type: Option<String>,
        card_id: Option<String>,
        player_ids: Vec<PlayerId>,
    },

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

    // ── Apothecary ────────────────────────────────────────────────────────────
    /// Choose the injury result when the apothecary rolls an alternative (CLIENT_APOTHECARY_CHOICE).
    /// Java: `ClientCommandApothecaryChoice` — carries PlayerState bitmask and SeriousInjury name.
    /// `player_state`: Java PlayerState bitmask (0 = keep original, non-zero = use alternative).
    /// `serious_injury`: alternative serious injury name (None = no serious injury).
    ApothecaryChoice { player_state: u32, serious_injury: Option<String> },

    // ── Misc ─────────────────────────────────────────────────────────────────
    /// Select a player from a list prompt.
    SelectPlayer { player_id: PlayerId },
    /// Select a skill when levelling up.
    SelectSkill { skill_id: SkillId },
    /// Change the weather (Weather Mage).
    SelectWeather { weather: Weather },
    /// Acknowledge an information-only dialog.
    Acknowledge,

    // ── Multi-block re-roll commands ──────────────────────────────────────────
    /// Use a re-roll for a specific target in a multi-block sequence.
    /// Java: `ClientCommandUseReRollForTarget`.
    UseReRollForTarget {
        re_rolled_action: Option<String>,
        re_roll_source: Option<String>,
        target_id: Option<String>,
    },
    /// Choose the Lord-of-Chaos player when using a single-use team re-roll.
    /// Java: `ClientCommandPlayerChoice(LORD_OF_CHAOS)`.
    LordOfChaosChoice { player_id: Option<String> },
    /// Choose the Indomitable target when multiple Dauntless targets exist.
    /// Java: `ClientCommandPlayerChoice(INDOMITABLE)`.
    IndomitableChoice { player_id: String },
    /// Player choice from a list dialog (generic — covers PlayerChoiceMode variants not yet specialised).
    /// Java: `ClientCommandPlayerChoice`.
    PlayerChoice { player_id: Option<String>, player_ids: Vec<String>, mode: String },

    // ── Blood lust ────────────────────────────────────────────────────────────
    /// BB2020: Vampire chose whether to change action after failing blood lust.
    /// Java: `ClientCommandBloodlustAction` — `change = true` means switch to alternate action to feed.
    BloodlustAction { change: bool },

    // ── Game lifecycle ────────────────────────────────────────────────────────
    /// Coach signals readiness to start the game.
    /// Java: `CLIENT_START_GAME` command. `home = true` → home coach, `false` → away coach.
    StartGame { home: bool },

    // ── Petty cash ────────────────────────────────────────────────────────────
    /// Coach chooses how much petty cash to spend on inducements.
    /// Java: `ClientCommandPettyCash` — `home` identifies which team's coach sent it.
    PettyCash { home: bool, amount: i32 },
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
    // Java: SpecialEffect.ZAP is also `isWizardSpell() == true` and is handled
    // identically to StepWizard's other two client-selectable spells.
    Zap,
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

    fn rt(a: Action) {
        let json = serde_json::to_string(&a).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back, "round-trip failed for {:?}", a);
    }

    #[test]
    fn play_card_with_target_round_trips() {
        rt(Action::PlayCard { card_id: "distract".into(), target_player_id: Some("p1".into()) });
    }

    #[test]
    fn play_card_without_target_round_trips() {
        rt(Action::PlayCard { card_id: "any".into(), target_player_id: None });
    }

    #[test]
    fn lash_out_round_trips() {
        rt(Action::LashOut { target_id: "p2".into() });
    }

    #[test]
    fn bite_round_trips() {
        rt(Action::Bite { target_id: "victim".into() });
    }

    #[test]
    fn armour_roll_attack_round_trips() {
        rt(Action::ArmourRollAttack { target_id: "t".into() });
    }

    #[test]
    fn throw_keg_round_trips() {
        use ffb_model::types::FieldCoordinate;
        rt(Action::ThrowKeg { coord: FieldCoordinate::new(12, 7) });
    }

    #[test]
    fn trickster_move_round_trips() {
        use ffb_model::types::FieldCoordinate;
        rt(Action::TricksterMove { coord: FieldCoordinate::new(13, 8) });
    }

    #[test]
    fn apothecary_choice_round_trips() {
        rt(Action::ApothecaryChoice { player_state: 0x100, serious_injury: Some("BrokenRibs".into()) });
    }

    #[test]
    fn apothecary_choice_no_si_round_trips() {
        rt(Action::ApothecaryChoice { player_state: 0, serious_injury: None });
    }

    #[test]
    fn use_inducement_with_type_round_trips() {
        rt(Action::UseInducement {
            inducement_type: Some("WIZARD".into()),
            card_id: None,
            player_ids: vec!["p1".into()],
        });
    }

    #[test]
    fn use_inducement_with_card_round_trips() {
        rt(Action::UseInducement {
            inducement_type: None,
            card_id: Some("distract".into()),
            player_ids: vec![],
        });
    }
}
