use crate::marking::player_marker::PlayerMarker;

/// 1:1 translation of `com.fumbbl.ffb.marking.TransientPlayerMarker`.
///
/// Client-side only marker — cannot be serialised or used for the away team.
#[derive(Debug, Clone)]
pub struct TransientPlayerMarker {
    pub base: PlayerMarker,
    pub mode: Mode,
}

/// Java inner enum `TransientPlayerMarker.Mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    APPEND,
    PREPEND,
    REPLACE,
    ADD,
}

impl Mode {
    pub fn get_display_text(&self) -> &'static str {
        match self {
            Mode::APPEND  => "Append",
            Mode::PREPEND => "Prepend",
            Mode::REPLACE => "Replace",
            Mode::ADD     => "Add",
        }
    }
}

impl TransientPlayerMarker {
    pub fn new(player_id: impl Into<String>, mode: Mode) -> Self {
        Self { base: PlayerMarker::with_player_id(player_id), mode }
    }

    pub fn get_mode(&self) -> Mode { self.mode }
    pub fn set_mode(&mut self, mode: Mode) { self.mode = mode; }
    pub fn get_player_id(&self) -> Option<&str> { self.base.get_player_id() }
    pub fn get_home_text(&self) -> Option<&str> { self.base.get_home_text() }
    pub fn set_home_text(&mut self, text: impl Into<String>) { self.base.set_home_text(text); }
}

impl Default for TransientPlayerMarker {
    fn default() -> Self {
        Self { base: PlayerMarker::new(), mode: Mode::REPLACE }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_display_text() {
        assert_eq!(Mode::APPEND.get_display_text(), "Append");
        assert_eq!(Mode::ADD.get_display_text(), "Add");
    }

    #[test]
    fn new_sets_player_id_and_mode() {
        let m = TransientPlayerMarker::new("p1", Mode::PREPEND);
        assert_eq!(m.get_player_id(), Some("p1"));
        assert_eq!(m.get_mode(), Mode::PREPEND);
    }
}
