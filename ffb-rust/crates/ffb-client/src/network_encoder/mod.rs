use ffb_engine::action::{Action, PlayerActionChoice};
use ffb_model::enums::PlayerAction;
use ffb_protocol::client_commands::*;

/// Convert a headless engine `Action` to the corresponding `ClientCommand`
/// for transmission to the Java server.
///
/// Returns `None` if the action has no network equivalent (e.g. `Acknowledge`).
pub fn encode(action: Action, active_player_id: Option<&str>) -> Option<ClientCommand> {
    match action {
        Action::CoinChoice { heads } => Some(ClientCommand::ClientCoinChoice(ClientCoinChoice {
            home_choice: heads,
        })),

        Action::ReceiveChoice { receive } => {
            Some(ClientCommand::ClientReceiveChoice(ClientReceiveChoice { receive }))
        }

        Action::PlacePlayer { player_id, coord } => {
            Some(ClientCommand::ClientSetupPlayer(ClientSetupPlayer {
                player_id,
                coordinate: coord,
            }))
        }

        Action::ConfirmSetup => Some(ClientCommand::ClientConfirm(ClientConfirm)),

        Action::KickBall { coord } => {
            Some(ClientCommand::ClientKickoff(ClientKickoff { coordinate: coord }))
        }

        Action::Touchback { player_id } => {
            Some(ClientCommand::ClientTouchback(ClientTouchback { player_id }))
        }

        Action::ActivatePlayer { player_id, player_action } => {
            let action = choice_to_player_action(player_action);
            Some(ClientCommand::ClientActingPlayer(ClientActingPlayer {
                player_id,
                player_action: action,
                standing_up: false,
            }))
        }

        Action::EndTurn => Some(ClientCommand::ClientEndTurn(ClientEndTurn)),

        Action::Move { path } => Some(ClientCommand::ClientMove(ClientMove {
            move_squares: path,
        })),

        Action::Block { defender_id } => {
            Some(ClientCommand::ClientBlock(ClientBlock { defender_id }))
        }

        Action::BlockChoice { die_index } => {
            Some(ClientCommand::ClientBlockChoice(ClientBlockChoice {
                selected_die_index: die_index as i32,
            }))
        }

        Action::PushTo { coord } => {
            Some(ClientCommand::ClientPushback(ClientPushback { pushback_square: coord }))
        }

        Action::FollowUp { follow_up } => {
            Some(ClientCommand::ClientFollowupChoice(ClientFollowupChoice { follow_up }))
        }

        Action::Pass { coord } => {
            Some(ClientCommand::ClientPass(ClientPass { target_coordinate: coord, hail_mary: false }))
        }

        Action::Intercept { attempt } => {
            Some(ClientCommand::ClientInterceptorChoice(ClientInterceptorChoice {
                attempt_interception: attempt,
            }))
        }

        Action::HandOff { receiver_id } => {
            Some(ClientCommand::ClientHandOver(ClientHandOver { target_player_id: receiver_id }))
        }

        Action::Foul { target_id } => {
            Some(ClientCommand::ClientFoul(ClientFoul { defender_id: target_id }))
        }

        Action::ThrowTeamMate { player_id, coord } => {
            Some(ClientCommand::ClientThrowTeamMate(ClientThrowTeamMate {
                player_id,
                target_coordinate: coord,
            }))
        }

        Action::KickTeamMate { player_id, coord } => {
            Some(ClientCommand::ClientKickTeamMate(ClientKickTeamMate {
                player_id,
                target_coordinate: coord,
            }))
        }

        Action::HypnoticGaze { target_id } => {
            Some(ClientCommand::ClientGaze(ClientGaze { target_id }))
        }

        Action::UseSkill { skill_id, use_skill } => {
            let pid = active_player_id.unwrap_or("").to_string();
            Some(ClientCommand::ClientUseSkill(ClientUseSkill {
                player_id: pid,
                skill: format!("{skill_id:?}"),
                use_skill,
            }))
        }

        Action::UseReRoll { use_reroll } => {
            Some(ClientCommand::ClientUseReRoll(ClientUseReRoll { use_reroll }))
        }

        Action::UseApothecary { player_id, use_apothecary } => {
            Some(ClientCommand::ClientUseApothecary(ClientUseApothecary {
                player_id,
                use_apothecary,
            }))
        }

        Action::UseBribe { use_bribe: _ } => {
            // Bribe is handled via confirm / no-op in the Java protocol
            Some(ClientCommand::ClientConfirm(ClientConfirm))
        }

        Action::ArgueTheCall { argue: _ } => {
            Some(ClientCommand::ClientConfirm(ClientConfirm))
        }

        Action::BuyInducements { purchases } => {
            let team_id = String::new(); // filled by caller if needed
            Some(ClientCommand::ClientBuyInducements(ClientBuyInducements {
                team_id,
                purchases: purchases
                    .into_iter()
                    .map(|p| (p.id, p.count as i32))
                    .collect(),
            }))
        }

        Action::SelectPlayer { player_id } => {
            Some(ClientCommand::ClientPlayerChoice(ClientPlayerChoice { player_id }))
        }

        Action::SelectWeather { weather } => {
            Some(ClientCommand::ClientSelectWeather(ClientSelectWeather {
                weather: format!("{weather:?}"),
            }))
        }

        Action::WizardSpell { spell, coord } => {
            Some(ClientCommand::ClientWizardSpell(ClientWizardSpell {
                team_id: String::new(),
                spell: format!("{spell:?}"),
                target_coordinate: Some(coord),
            }))
        }

        Action::ThrowBomb { coord } => {
            // Throw bomb is a pass-type action
            Some(ClientCommand::ClientPass(ClientPass { target_coordinate: coord, hail_mary: false }))
        }

        Action::Punt { coord } => {
            // Punt uses the kickoff command (server maps it based on player action context)
            Some(ClientCommand::ClientKickoff(ClientKickoff { coordinate: coord }))
        }

        Action::SelectSkill { .. } => Some(ClientCommand::ClientConfirm(ClientConfirm)),

        Action::PlayCard { card_id: _ } => Some(ClientCommand::ClientConfirm(ClientConfirm)),

        Action::Acknowledge => None,

        Action::MultiBlock { defender1_id, defender2_id: _ } => {
            // Use first defender for the primary block command; server handles both
            Some(ClientCommand::ClientBlock(ClientBlock { defender_id: defender1_id }))
        }

        Action::Stab { defender_id } => {
            // Stab uses the same ClientBlock command as a regular block
            Some(ClientCommand::ClientBlock(ClientBlock { defender_id }))
        }

        Action::BreatheFire { target_id } => {
            // BreatheFire targets an adjacent opponent (maps to block command)
            Some(ClientCommand::ClientBlock(ClientBlock { defender_id: target_id }))
        }

        Action::ProjectileVomit { target_id } => {
            Some(ClientCommand::ClientBlock(ClientBlock { defender_id: target_id }))
        }

        Action::HitAndRun { coord } => {
            // HitAndRun move uses the move command with the chosen square, or confirm if declining
            match coord {
                Some(c) => Some(ClientCommand::ClientMove(ClientMove { move_squares: vec![c] })),
                None => Some(ClientCommand::ClientConfirm(ClientConfirm)),
            }
        }

        // Star-player special attacks — map to closest existing protocol commands
        Action::LashOut { target_id } => Some(ClientCommand::ClientBlock(ClientBlock { defender_id: target_id })),
        Action::Bite { target_id } => Some(ClientCommand::ClientBlock(ClientBlock { defender_id: target_id })),
        Action::ArmourRollAttack { target_id } => Some(ClientCommand::ClientBlock(ClientBlock { defender_id: target_id })),
        Action::ThrowKeg { coord } => Some(ClientCommand::ClientPass(ClientPass { target_coordinate: coord, hail_mary: false })),
        Action::CatchOfTheDay => Some(ClientCommand::ClientConfirm(ClientConfirm)),

        Action::TricksterMove { coord } => {
            Some(ClientCommand::ClientMove(ClientMove { move_squares: vec![coord] }))
        }
    }
}

fn choice_to_player_action(choice: PlayerActionChoice) -> PlayerAction {
    match choice {
        PlayerActionChoice::Move => PlayerAction::Move,
        PlayerActionChoice::Blitz => PlayerAction::Blitz,
        PlayerActionChoice::Block => PlayerAction::Block,
        PlayerActionChoice::Stab => PlayerAction::Stab,
        PlayerActionChoice::Foul => PlayerAction::Foul,
        PlayerActionChoice::Pass => PlayerAction::Pass,
        PlayerActionChoice::HandOff => PlayerAction::HandOver,
        PlayerActionChoice::ThrowTeamMate => PlayerAction::ThrowTeamMate,
        PlayerActionChoice::KickTeamMate => PlayerAction::KickTeamMate,
        PlayerActionChoice::HypnoticGaze => PlayerAction::Gaze,
        PlayerActionChoice::ThrowBomb => PlayerAction::ThrowBomb,
        PlayerActionChoice::Swoop => PlayerAction::Swoop,
        PlayerActionChoice::Punt => PlayerAction::Punt,
        PlayerActionChoice::StandUp => PlayerAction::StandUp,
        PlayerActionChoice::StandUpBlitz => PlayerAction::StandUpBlitz,
        PlayerActionChoice::BreatheFire => PlayerAction::BreatheFire,
        PlayerActionChoice::ProjectileVomit => PlayerAction::ProjectileVomit,
        PlayerActionChoice::SecureTheBall => PlayerAction::SecureTheBall,
    }
}
