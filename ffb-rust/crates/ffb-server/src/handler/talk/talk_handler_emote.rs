/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerEmote.
/// Handles spectator emote chat commands (aah, boo, cheer, clap, crickets, etc.).
///
/// Java's constructor uses `com.fumbbl.ffb.ChatCommand.effectsAsStrings()` — the
/// ten emote command strings (excluding `/specs`, whose `effect` flag is false).
/// There is no ported Rust equivalent of that enum (`ffb_model::model::chat_command`
/// is a different Java class — a chat-log entry, not this command enum), so the
/// command set and command→`SoundId` mapping are inlined here directly from
/// `ChatCommand.java`'s literal table.
///
/// Java's `playSoundAfterCooldown(server, gameState, coach, sound)` sends the
/// sound immediately via `communication.sendSound`; the base
/// `TalkHandler::play_sound_after_cooldown` (already ported) instead returns
/// the new cooldown timestamp to persist. This handler combines that with the
/// resolved `SoundId` and returns `Some((sound, new_cooldown_time))` for the
/// caller to actually dispatch once outbound-send infra exists (documented the
/// same way as `talk_handler_activated.rs`).
use std::collections::HashSet;
use ffb_model::model::sound_id::SoundId;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment};

/// Java: `ChatCommand.effectsAsStrings()` — the ten emote command strings.
const EFFECT_COMMANDS: [&str; 10] = [
    "/aah", "/boo", "/cheer", "/clap", "/crickets",
    "/hurt", "/laugh", "/ooh", "/shock", "/stomp",
];

pub struct TalkHandlerEmote {
    base: TalkHandler,
}

impl TalkHandlerEmote {
    /// Java: `TalkHandlerEmote()`.
    pub fn new() -> Self {
        let mut commands = HashSet::new();
        for c in EFFECT_COMMANDS {
            commands.insert(c.to_string());
        }
        Self {
            base: TalkHandler::new(commands, 0, Client::Spec, Environment::None, HashSet::new()),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `ChatCommand.fromCommand(command)` + the `switch (chatCommand)` dispatch —
    /// maps the command string to its associated spectator `SoundId`.
    fn sound_for_command(command: &str) -> Option<SoundId> {
        match command {
            "/aah" => Some(SoundId::SPEC_AAH),
            "/boo" => Some(SoundId::SPEC_BOO),
            "/cheer" => Some(SoundId::SPEC_CHEER),
            "/clap" => Some(SoundId::SPEC_CLAP),
            "/crickets" => Some(SoundId::SPEC_CRICKETS),
            "/hurt" => Some(SoundId::SPEC_HURT),
            "/laugh" => Some(SoundId::SPEC_LAUGH),
            "/ooh" => Some(SoundId::SPEC_OOH),
            "/shock" => Some(SoundId::SPEC_SHOCK),
            "/stomp" => Some(SoundId::SPEC_STOMP),
            _ => None,
        }
    }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// resolves the emote sound and applies the spectator cooldown. `coach` is
    /// Java's `server.getSessionManager().getCoachForSession(session)`; returns
    /// `None` when either the command is unrecognized, the coach session is
    /// unknown, or the cooldown blocks the sound (matching Java's silent
    /// no-op paths). On success, returns the sound to play plus the new
    /// cooldown timestamp to persist.
    pub fn handle(
        &self,
        coach: Option<&str>,
        commands: &[String],
        last_cooldown_time: i64,
        spectator_cooldown_ms: Option<i64>,
        current_time_ms: i64,
    ) -> Option<(SoundId, i64)> {
        let command = commands.first()?;
        let sound = Self::sound_for_command(command)?;
        let _coach = coach?;
        let new_cooldown = self.base.play_sound_after_cooldown(last_cooldown_time, spectator_cooldown_ms, current_time_ms)?;
        Some((sound, new_cooldown))
    }
}

impl Default for TalkHandlerEmote {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() { let _ = TalkHandlerEmote::new(); }

    #[test]
    fn handle_returns_sound_and_cooldown_for_known_command() {
        let h = TalkHandlerEmote::new();
        let commands = vec!["/cheer".to_string()];
        let result = h.handle(Some("Alice"), &commands, 0, Some(5000), 7000);
        assert_eq!(result, Some((SoundId::SPEC_CHEER, 7000)));
    }

    #[test]
    fn handle_returns_none_for_unknown_command() {
        let h = TalkHandlerEmote::new();
        let commands = vec!["/dance".to_string()];
        let result = h.handle(Some("Alice"), &commands, 0, Some(5000), 7000);
        assert!(result.is_none());
    }

    #[test]
    fn handle_returns_none_when_coach_unknown() {
        let h = TalkHandlerEmote::new();
        let commands = vec!["/boo".to_string()];
        let result = h.handle(None, &commands, 0, Some(5000), 7000);
        assert!(result.is_none());
    }

    #[test]
    fn handle_blocked_by_cooldown() {
        let h = TalkHandlerEmote::new();
        let commands = vec!["/aah".to_string()];
        let result = h.handle(Some("Alice"), &commands, 1000, Some(5000), 3000);
        assert!(result.is_none());
    }

    #[test]
    fn handle_plays_when_no_cooldown_configured() {
        let h = TalkHandlerEmote::new();
        let commands = vec!["/stomp".to_string()];
        let result = h.handle(Some("Alice"), &commands, 1000, None, 3000);
        assert_eq!(result, Some((SoundId::SPEC_STOMP, 1000)));
    }
}
