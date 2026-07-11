use serde::{Deserialize, Serialize};

/// The scalar type of data carried by a `ModelChange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelChangeDataType {
    Null,
    Boolean,
    String,
    PlayerAction,
    Skill,
    Long,
    Date,
    TurnMode,
    FieldCoordinate,
    DialogId,
    DialogParameter,
    Integer,
    PlayerState,
    SeriousInjury,
    SendToBoxReason,
    BloodSpot,
    TrackNumber,
    PushbackSquare,
    MoveSquare,
    Weather,
    RangeRuler,
    DiceDecoration,
    Inducement,
    FieldMarker,
    PlayerMarker,
    GameOption,
    Card,
    LeaderState,
    CardEffect,
    CardChoices,
    BlitzState,
    TargetSelectionState,
    Prayer,
    TrapDoor,
    SketchState,
}

impl ModelChangeDataType {
    pub fn for_name(name: &str) -> Option<Self> {
        serde_json::from_str(&format!("\"{}\"", name)).ok()
    }

    pub fn name(self) -> &'static str {
        match self {
            ModelChangeDataType::Null => "null",
            ModelChangeDataType::Boolean => "boolean",
            ModelChangeDataType::String => "string",
            ModelChangeDataType::PlayerAction => "playerAction",
            ModelChangeDataType::Skill => "skill",
            ModelChangeDataType::Long => "long",
            ModelChangeDataType::Date => "date",
            ModelChangeDataType::TurnMode => "turnMode",
            ModelChangeDataType::FieldCoordinate => "fieldCoordinate",
            ModelChangeDataType::DialogId => "dialogId",
            ModelChangeDataType::DialogParameter => "dialogParameter",
            ModelChangeDataType::Integer => "integer",
            ModelChangeDataType::PlayerState => "playerState",
            ModelChangeDataType::SeriousInjury => "seriousInjury",
            ModelChangeDataType::SendToBoxReason => "sendToBoxReason",
            ModelChangeDataType::BloodSpot => "bloodSpot",
            ModelChangeDataType::TrackNumber => "trackNumber",
            ModelChangeDataType::PushbackSquare => "pushbackSquare",
            ModelChangeDataType::MoveSquare => "moveSquare",
            ModelChangeDataType::Weather => "weather",
            ModelChangeDataType::RangeRuler => "rangeRuler",
            ModelChangeDataType::DiceDecoration => "diceDecoration",
            ModelChangeDataType::Inducement => "inducement",
            ModelChangeDataType::FieldMarker => "fieldMarker",
            ModelChangeDataType::PlayerMarker => "playerMarker",
            ModelChangeDataType::GameOption => "gameOption",
            ModelChangeDataType::Card => "card",
            ModelChangeDataType::LeaderState => "leaderState",
            ModelChangeDataType::CardEffect => "cardEffect",
            ModelChangeDataType::CardChoices => "cardChoices",
            ModelChangeDataType::BlitzState => "blitzState",
            ModelChangeDataType::TargetSelectionState => "targetSelectionState",
            ModelChangeDataType::Prayer => "prayer",
            ModelChangeDataType::TrapDoor => "trapDoor",
            ModelChangeDataType::SketchState => "sketchState",
        }
    }
}

/// Identifies a specific field change within the game model.
/// Used for incremental sync between server and client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelChangeId {
    // ActingPlayer fields
    ActingPlayerMarkSkillUsed,
    ActingPlayerMarkSkillUnused,
    ActingPlayerSetCurrentMove,
    ActingPlayerSetDodging,
    ActingPlayerSetGoingForIt,
    ActingPlayerSetHasBlocked,
    ActingPlayerSetHasFed,
    ActingPlayerSetHasFouled,
    ActingPlayerSetHasJumped,
    ActingPlayerSetHasMoved,
    ActingPlayerSetHasPassed,
    ActingPlayerSetHasTriggeredEffect,
    ActingPlayerSetJumping,
    ActingPlayerSetOldPlayerState,
    ActingPlayerSetPlayerAction,
    ActingPlayerSetPlayerId,
    ActingPlayerSetStandingUp,
    ActingPlayerSetStrength,
    ActingPlayerSetSufferingAnimosity,
    ActingPlayerSetSufferingBloodLust,
    ActingPlayerSetJumpsWithoutModifiers,
    ActingPlayerSetHeldInPlace,
    ActingPlayerSetMustCompleteAction,
    ActingPlayerSetFellFromRush,

    // FieldModel fields
    FieldModelAddBloodSpot,
    FieldModelAddCard,
    FieldModelAddCardEffect,
    FieldModelAddDiceDecoration,
    FieldModelAddEnhancements,
    FieldModelAddFieldMarker,
    FieldModelAddHatred,
    FieldModelAddIntensiveTraining,
    FieldModelAddMoveSquare,
    FieldModelAddPlayerMarker,
    FieldModelAddPrayer,
    FieldModelAddPushbackSquare,
    FieldModelAddSkillEnhancements,
    FieldModelAddTrackNumber,
    FieldModelAddTrapDoor,
    FieldModelAddWisdom,
    FieldModelKeepDeactivatedCard,
    FieldModelRemoveCard,
    FieldModelRemoveCardEffect,
    FieldModelRemoveDiceDecoration,
    FieldModelRemoveFieldMarker,
    FieldModelRemoveMoveSquare,
    FieldModelRemovePlayer,
    FieldModelRemovePlayerMarker,
    FieldModelRemovePrayer,
    FieldModelRemovePushbackSquare,
    FieldModelRemoveTrackNumber,
    FieldModelRemoveTrapDoor,
    FieldModelSetBallCoordinate,
    FieldModelSetBallInPlay,
    FieldModelSetBallLost,
    FieldModelSetBombCoordinate,
    FieldModelSetHomeTeam,
    FieldModelSetPlayerCoordinate,
    FieldModelSetPlayerState,
    FieldModelSetRangeRuler,
    FieldModelSetWeather,

    // Game fields
    GameSetActingTeam,
    GameSetDialogParameter,
    GameSetFinished,
    GameSetFirstHalf,
    GameSetFirstOffense,
    GameSetGameResult,
    GameSetHomeFirstOffense,
    GameSetLastTurnMode,
    GameSetPassCoordinate,
    GameSetSetupOffense,
    GameSetStatus,
    GameSetThrowerCoordinate,
    GameSetThrowerPlayerId,
    GameSetTurnMode,
    GameSetWaitingForOpponent,
    GameOptionsAddOption,

    // InducementSet fields
    InducementSetActivateCard,
    InducementSetAddAvailableCard,
    InducementSetAddInducement,
    InducementSetCardChoices,
    InducementSetDeactivateCard,
    InducementSetAddPrayer,
    InducementSetRemoveAvailableCard,
    InducementSetRemoveInducement,
    InducementSetRemovePrayer,

    // Player fields
    PlayerMarkSkillUsed,
    PlayerMarkSkillUnused,

    // PlayerResult fields
    PlayerResultSetBlocks,
    PlayerResultSetCasualties,
    PlayerResultSetCasualtiesWithAdditionalSpp,
    PlayerResultSetCatchesWithAdditionalSpp,
    PlayerResultSetCompletions,
    PlayerResultSetCompletionsWithAdditionalSpp,
    PlayerResultSetCurrentSpps,
    PlayerResultSetDefecting,
    PlayerResultSetFouls,
    PlayerResultSetHasUsedSecretWeapon,
    PlayerResultSetInterceptions,
    PlayerResultSetDeflections,
    PlayerResultSetPassing,
    PlayerResultSetPlayerAwards,
    PlayerResultSetRushing,
    PlayerResultSetSendToBoxByPlayerId,
    PlayerResultSetSendToBoxHalf,
    PlayerResultSetSendToBoxReason,
    PlayerResultSetSendToBoxTurn,
    PlayerResultSetSeriousInjury,
    PlayerResultSetSeriousInjuryDecay,
    PlayerResultSetTouchdowns,
    PlayerResultSetTurnsPlayed,
    PlayerResultSetLandings,

    // Sketch
    SketchUpdate,

    // TargetSelection
    TargetSelectionCommitted,

    // TeamResult fields
    TeamResultSetConceded,
    TeamResultSetDedicatedFansModifier,
    TeamResultSetFame,
    TeamResultSetFanFactor,
    TeamResultSetBadlyHurtSuffered,
    TeamResultSetFanFactorModifier,
    TeamResultSetPenaltyScore,
    TeamResultSetPettyCashTransferred,
    TeamResultSetPettyCashUsed,
    TeamResultSetRaisedDead,
    TeamResultSetRipSuffered,
    TeamResultSetScore,
    TeamResultSetSeriousInjurySuffered,
    TeamResultSetSpectators,
    TeamResultSetSpirallingExpenses,
    TeamResultSetTeamValue,
    TeamResultSetWinnings,

    // TurnData fields
    TurnDataSetApothecaries,
    TurnDataSetBlitzUsed,
    TurnDataSetBombUsed,
    TurnDataSetFirstTurnAfterKickoff,
    TurnDataSetFoulUsed,
    TurnDataSetHandOverUsed,
    TurnDataSetLeaderState,
    TurnDataSetPassUsed,
    TurnDataSetPlagueDoctors,
    TurnDataSetTtmUsed,
    TurnDataSetKtmUsed,
    TurnDataSetSecureTheBallUsed,
    TurnDataSetPuntUsed,
    TurnDataSetReRolls,
    TurnDataSetReRollsBrilliantCoachingOneDrive,
    TurnDataSetReRollsPumpUpTheCrowdOneDrive,
    TurnDataSetReRollsShowStarOneDrive,
    TurnDataSetReRollsSingleUse,
    TurnDataSetReRollUsed,
    TurnDataSetTurnNr,
    TurnDataSetTurnStarted,
    TurnDataSetWanderingApothecaries,
    TurnDataSetCoachBanned,
}

impl ModelChangeId {
    pub fn get_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap_or_default()
            .trim_matches('"')
            .to_owned()
    }

    pub fn for_name(name: &str) -> Option<Self> {
        serde_json::from_str(&format!("\"{}\"", name)).ok()
    }

    pub fn data_type(self) -> ModelChangeDataType {
        match self {
            ModelChangeId::ActingPlayerMarkSkillUsed
            | ModelChangeId::ActingPlayerMarkSkillUnused
            | ModelChangeId::ActingPlayerSetHasTriggeredEffect
            | ModelChangeId::ActingPlayerSetJumping
            | ModelChangeId::ActingPlayerSetStandingUp
            | ModelChangeId::ActingPlayerSetSufferingAnimosity
            | ModelChangeId::ActingPlayerSetSufferingBloodLust
            | ModelChangeId::ActingPlayerSetJumpsWithoutModifiers
            | ModelChangeId::ActingPlayerSetHeldInPlace
            | ModelChangeId::ActingPlayerSetMustCompleteAction
            | ModelChangeId::PlayerMarkSkillUsed
            | ModelChangeId::PlayerMarkSkillUnused
            | ModelChangeId::FieldModelAddIntensiveTraining
            | ModelChangeId::FieldModelAddWisdom => ModelChangeDataType::Skill,

            ModelChangeId::ActingPlayerSetCurrentMove
            | ModelChangeId::ActingPlayerSetStrength
            | ModelChangeId::PlayerResultSetBlocks
            | ModelChangeId::PlayerResultSetCasualties
            | ModelChangeId::TurnDataSetApothecaries
            | ModelChangeId::TurnDataSetReRolls
            | ModelChangeId::TurnDataSetTurnNr => ModelChangeDataType::Integer,

            ModelChangeId::ActingPlayerSetDodging
            | ModelChangeId::ActingPlayerSetGoingForIt
            | ModelChangeId::ActingPlayerSetHasBlocked
            | ModelChangeId::ActingPlayerSetHasFed
            | ModelChangeId::ActingPlayerSetHasFouled
            | ModelChangeId::ActingPlayerSetHasJumped
            | ModelChangeId::ActingPlayerSetHasMoved
            | ModelChangeId::ActingPlayerSetHasPassed
            | ModelChangeId::ActingPlayerSetFellFromRush
            | ModelChangeId::TurnDataSetBlitzUsed
            | ModelChangeId::TurnDataSetReRollUsed
            | ModelChangeId::TurnDataSetTurnStarted
            | ModelChangeId::TargetSelectionCommitted
            | ModelChangeId::GameSetFinished
            | ModelChangeId::TeamResultSetConceded => ModelChangeDataType::Boolean,

            ModelChangeId::ActingPlayerSetPlayerAction => ModelChangeDataType::PlayerAction,
            ModelChangeId::ActingPlayerSetPlayerId => ModelChangeDataType::String,
            ModelChangeId::ActingPlayerSetOldPlayerState
            | ModelChangeId::FieldModelSetPlayerState => ModelChangeDataType::PlayerState,

            ModelChangeId::FieldModelAddBloodSpot => ModelChangeDataType::BloodSpot,
            ModelChangeId::FieldModelAddCard
            | ModelChangeId::FieldModelKeepDeactivatedCard
            | ModelChangeId::FieldModelRemoveCard
            | ModelChangeId::InducementSetActivateCard
            | ModelChangeId::InducementSetDeactivateCard => ModelChangeDataType::Card,
            ModelChangeId::FieldModelAddCardEffect
            | ModelChangeId::FieldModelRemoveCardEffect => ModelChangeDataType::CardEffect,
            ModelChangeId::FieldModelAddDiceDecoration
            | ModelChangeId::FieldModelRemoveDiceDecoration => ModelChangeDataType::DiceDecoration,
            ModelChangeId::FieldModelAddMoveSquare
            | ModelChangeId::FieldModelRemoveMoveSquare => ModelChangeDataType::MoveSquare,
            ModelChangeId::FieldModelAddPushbackSquare
            | ModelChangeId::FieldModelRemovePushbackSquare => ModelChangeDataType::PushbackSquare,
            ModelChangeId::FieldModelSetBallCoordinate
            | ModelChangeId::FieldModelSetPlayerCoordinate
            | ModelChangeId::FieldModelSetBombCoordinate
            | ModelChangeId::FieldModelRemovePlayer => ModelChangeDataType::FieldCoordinate,
            ModelChangeId::FieldModelSetWeather => ModelChangeDataType::Weather,
            ModelChangeId::GameSetTurnMode | ModelChangeId::GameSetLastTurnMode => {
                ModelChangeDataType::TurnMode
            }
            ModelChangeId::PlayerResultSetSendToBoxReason => ModelChangeDataType::SendToBoxReason,
            ModelChangeId::PlayerResultSetSeriousInjury
            | ModelChangeId::PlayerResultSetSeriousInjuryDecay => ModelChangeDataType::SeriousInjury,
            ModelChangeId::TurnDataSetLeaderState => ModelChangeDataType::LeaderState,
            ModelChangeId::InducementSetAddPrayer
            | ModelChangeId::InducementSetRemovePrayer => ModelChangeDataType::Prayer,
            ModelChangeId::InducementSetAddInducement
            | ModelChangeId::InducementSetRemoveInducement => ModelChangeDataType::Inducement,
            ModelChangeId::InducementSetCardChoices => ModelChangeDataType::CardChoices,
            ModelChangeId::SketchUpdate => ModelChangeDataType::SketchState,
            _ => ModelChangeDataType::String,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_data_type() {
        let dt = ModelChangeDataType::PlayerState;
        let json = serde_json::to_string(&dt).unwrap();
        let back: ModelChangeDataType = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, back);
    }

    #[test]
    fn data_type_for_skill_change() {
        assert_eq!(
            ModelChangeId::ActingPlayerMarkSkillUsed.data_type(),
            ModelChangeDataType::Skill
        );
    }

    #[test]
    fn model_change_id_for_name_field_model_set_weather() {
        assert_eq!(
            ModelChangeId::for_name("fieldModelSetWeather"),
            Some(ModelChangeId::FieldModelSetWeather)
        );
    }

    #[test]
    fn model_change_id_get_name_round_trips() {
        let id = ModelChangeId::TurnDataSetTurnNr;
        let name = id.get_name();
        assert_eq!(ModelChangeId::for_name(&name), Some(id));
    }

    #[test]
    fn model_change_data_type_name_method() {
        assert_eq!(ModelChangeDataType::Skill.name(), "skill");
        assert_eq!(ModelChangeDataType::Integer.name(), "integer");
        assert_eq!(ModelChangeDataType::Boolean.name(), "boolean");
    }

    #[test]
    fn model_change_data_type_for_name_round_trip() {
        let dt = ModelChangeDataType::TurnMode;
        assert_eq!(ModelChangeDataType::for_name("turnMode"), Some(dt));
    }

    #[test]
    fn model_change_id_for_name_unknown_returns_none() {
        assert_eq!(ModelChangeId::for_name("notAField"), None);
    }

    #[test]
    fn model_change_data_type_serde_null() {
        let dt = ModelChangeDataType::Null;
        let json = serde_json::to_string(&dt).unwrap();
        let back: ModelChangeDataType = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, back);
    }
}
