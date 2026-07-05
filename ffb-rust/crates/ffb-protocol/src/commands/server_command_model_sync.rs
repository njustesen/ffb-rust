use ffb_model::model::animation::Animation;
use ffb_model::model::change::ModelChangeList;
use ffb_model::model::report_list::ReportList;
use ffb_model::model::SoundId;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandModelSync`.
/// Sends a batch of model changes, reports, an animation, and sound to the client.
#[derive(Debug)]
pub struct ServerCommandModelSync {
    /// Java: `fModelChanges` — list of model state deltas.
    pub model_changes: ModelChangeList,
    /// Java: `fReportList` — list of game reports to display.
    pub report_list: ReportList,
    /// Java: `fAnimation` — animation to play on the client.
    pub animation: Animation,
    /// Java: `fSound` — sound to play.
    pub sound: SoundId,
    /// Java: `fGameTime` — elapsed game clock in ms.
    pub game_time: i64,
    /// Java: `fTurnTime` — elapsed turn clock in ms.
    pub turn_time: i64,
}

impl ServerCommandModelSync {
    pub fn new(
        model_changes: ModelChangeList,
        report_list: ReportList,
        animation: Animation,
        sound: SoundId,
        game_time: i64,
        turn_time: i64,
    ) -> Self {
        Self { model_changes, report_list, animation, sound, game_time, turn_time }
    }
    pub fn get_model_changes(&self) -> &ModelChangeList { &self.model_changes }
    pub fn get_report_list(&self) -> &ReportList { &self.report_list }
    pub fn get_animation(&self) -> &Animation { &self.animation }
    pub fn get_sound(&self) -> SoundId { self.sound }
    pub fn get_game_time(&self) -> i64 { self.game_time }
    pub fn get_turn_time(&self) -> i64 { self.turn_time }
}

impl Default for ServerCommandModelSync {
    fn default() -> Self {
        Self {
            model_changes: ModelChangeList::default(),
            report_list: ReportList::default(),
            animation: Animation::default(),
            sound: SoundId::TOUCHDOWN,
            game_time: 0,
            turn_time: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandModelSync::new(
            ModelChangeList::default(),
            ReportList::default(),
            Animation::default(),
            SoundId::TOUCHDOWN,
            5000,
            2000,
        );
        assert_eq!(cmd.get_sound(), SoundId::TOUCHDOWN);
        assert_eq!(cmd.get_game_time(), 5000);
        assert_eq!(cmd.get_turn_time(), 2000);
    }

    #[test]
    fn default_zero_times() {
        let cmd = ServerCommandModelSync::default();
        assert_eq!(cmd.game_time, 0);
        assert_eq!(cmd.turn_time, 0);
    }
}
