use serde::{Deserialize, Serialize};
use crate::enums::InducementDuration;
use crate::inducement::card_target::CardTarget;

/// 1:1 translation of `com.fumbbl.ffb.inducement.Card`.
/// Represents a single inducement card with its handler key and properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    /// Name of the CardHandlerKey variant that handles this card (matches Java handlerKey().name()).
    pub handler_key_name: Option<String>,
    /// Java: Card.getDuration() — how long the card effect lasts.
    pub duration: Option<InducementDuration>,
    /// Java: Card.isRemainsInPlay() — card stays deactivated on field rather than being removed.
    pub remains_in_play: bool,
    /// Java: `Card.getTarget()` — TURN (played on the turn, no player selection needed) or one of
    /// the player-targeted variants. Defaults to `TURN`, matching most cards.
    #[serde(default)]
    pub target: CardTarget,
    /// Java: `Card.requiresBlockablePlayerSelection()` — overridden per-card-subclass in Java
    /// (e.g. Custard Pie); default `false`. No card catalog with per-card overrides exists yet
    /// in this port (cards are constructed ad hoc, not data-driven) — this field lets a
    /// specific `Card` instance opt in once a caller knows which card it's building.
    #[serde(default)]
    pub requires_blockable_player_selection: bool,
}

impl Card {
    pub fn new(name: impl Into<String>, handler_key_name: Option<impl Into<String>>) -> Self {
        Card {
            name: name.into(),
            handler_key_name: handler_key_name.map(|s| s.into()),
            duration: None,
            remains_in_play: false,
            target: CardTarget::default(),
            requires_blockable_player_selection: false,
        }
    }

    /// Builder: set the duration.
    pub fn with_duration(mut self, duration: InducementDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Builder: set remains_in_play.
    pub fn with_remains_in_play(mut self, remains: bool) -> Self {
        self.remains_in_play = remains;
        self
    }

    /// Builder: set the card's target.
    pub fn with_target(mut self, target: CardTarget) -> Self {
        self.target = target;
        self
    }

    /// Builder: set requires_blockable_player_selection.
    pub fn with_requires_blockable_player_selection(mut self, requires: bool) -> Self {
        self.requires_blockable_player_selection = requires;
        self
    }

    /// Java: Card.getName()
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Java: Card.handlerKey() — returns the handler key name if one is set.
    pub fn handler_key_name(&self) -> Option<&str> {
        self.handler_key_name.as_deref()
    }

    /// Java: Card.getDuration()
    pub fn get_duration(&self) -> Option<InducementDuration> {
        self.duration
    }

    /// Java: Card.isRemainsInPlay()
    pub fn is_remains_in_play(&self) -> bool {
        self.remains_in_play
    }

    /// Java: Card.getTarget()
    pub fn get_target(&self) -> CardTarget {
        self.target
    }

    /// Java: Card.requiresBlockablePlayerSelection()
    pub fn requires_blockable_player_selection(&self) -> bool {
        self.requires_blockable_player_selection
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new("", None::<&str>)
    }
}
