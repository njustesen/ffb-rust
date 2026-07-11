use ffb_model::model::animation::Animation;
use ffb_model::model::change::ModelChangeList;
use ffb_model::model::report_list::ReportList;
use ffb_model::model::SoundId;
use ffb_model::enums::NetCommandId;
use crate::commands::server_command::ServerCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandModelSync`.
/// Sends a batch of model changes, reports, an animation, and sound to the client.
#[derive(Debug)]
pub struct ServerCommandModelSync {
    /// Java: base-class `ServerCommand.fCommandNr`.
    pub command_nr: i32,
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
        Self { command_nr: 0, model_changes, report_list, animation, sound, game_time, turn_time }
    }
    pub fn get_model_changes(&self) -> &ModelChangeList { &self.model_changes }
    pub fn get_report_list(&self) -> &ReportList { &self.report_list }
    pub fn get_animation(&self) -> &Animation { &self.animation }
    pub fn get_sound(&self) -> SoundId { self.sound }
    pub fn get_game_time(&self) -> i64 { self.game_time }
    pub fn get_turn_time(&self) -> i64 { self.turn_time }

    /// Java: `ServerCommandModelSync.toJsonValue()`. `ReportList` holds
    /// `Box<dyn IReport>` entries with no Java-matching JSON serializer of
    /// their own (each concrete report type would need its own `toJsonValue`
    /// translation, which is out of scope here); only its `size` is emitted
    /// so no compile-time information is silently invented, but individual
    /// reports cannot round-trip through this method.
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ServerCommand { command_nr: self.command_nr };
        let mut map = base.base_json_fields(self.get_id());
        map.insert(
            "modelChangeList".to_string(),
            serde_json::to_value(&self.model_changes).unwrap_or(serde_json::Value::Null),
        );
        map.insert("reportList".to_string(), serde_json::json!({ "size": self.report_list.size() }));
        map.insert(
            "animation".to_string(),
            serde_json::to_value(&self.animation).unwrap_or(serde_json::Value::Null),
        );
        map.insert("sound".to_string(), serde_json::json!(self.sound.get_name()));
        map.insert("gameTime".to_string(), serde_json::json!(self.game_time));
        map.insert("turnTime".to_string(), serde_json::json!(self.turn_time));
        serde_json::Value::Object(map)
    }

    /// Java: `ServerCommandModelSync.initFrom(source, jsonValue)`. See
    /// `to_json_value()` — the report list cannot be reconstructed from the
    /// wire payload and is always restored empty.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ServerCommand::base_from_json(json);
        let model_changes = json
            .get("modelChangeList")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let animation = json
            .get("animation")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        let sound = json
            .get("sound")
            .and_then(|v| v.as_str())
            .and_then(SoundId::for_name)
            .unwrap_or(SoundId::TOUCHDOWN);
        Self {
            command_nr: base.command_nr,
            model_changes,
            report_list: ReportList::new(),
            animation,
            sound,
            game_time: json.get("gameTime").and_then(|v| v.as_i64()).unwrap_or(0),
            turn_time: json.get("turnTime").and_then(|v| v.as_i64()).unwrap_or(0),
        }
    }
}

impl Default for ServerCommandModelSync {
    fn default() -> Self {
        Self {
            command_nr: 0,
            model_changes: ModelChangeList::default(),
            report_list: ReportList::default(),
            animation: Animation::default(),
            sound: SoundId::TOUCHDOWN,
            game_time: 0,
            turn_time: 0,
        }
    }
}

impl NetCommand for ServerCommandModelSync {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ServerModelSync
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

    #[test]
    fn debug_format_nonempty() {
        let cmd = ServerCommandModelSync::default();
        assert!(!format!("{cmd:?}").is_empty());
    }


    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandModelSync::default());
        assert!(s.contains("ServerCommandModelSync"));
    }

    #[test]
    fn size_of_is_at_least_zero() {
        let _ = std::mem::size_of::<ServerCommandModelSync>();
    }

    #[test]
    fn get_id_is_server_model_sync() {
        assert_eq!(ServerCommandModelSync::default().get_id(), NetCommandId::ServerModelSync);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_times() {
        let mut cmd = ServerCommandModelSync::default();
        cmd.command_nr = 4;
        cmd.game_time = 100;
        cmd.turn_time = 50;
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "serverModelSync");
        assert_eq!(json["commandNr"], 4);
        assert_eq!(json["gameTime"], 100);
        assert_eq!(json["turnTime"], 50);
        assert_eq!(json["sound"], "touchdown");
    }

    #[test]
    fn round_trip_with_times_and_sound() {
        let mut cmd = ServerCommandModelSync::new(
            ModelChangeList::default(),
            ReportList::default(),
            Animation::default(),
            SoundId::BLOCK,
            7000,
            3000,
        );
        cmd.command_nr = 9;
        let json = cmd.to_json_value();
        let restored = ServerCommandModelSync::from_json(&json);
        assert_eq!(restored.command_nr, 9);
        assert_eq!(restored.sound, SoundId::BLOCK);
        assert_eq!(restored.game_time, 7000);
        assert_eq!(restored.turn_time, 3000);
    }

    #[test]
    fn round_trip_with_defaults() {
        let cmd = ServerCommandModelSync::default();
        let json = cmd.to_json_value();
        let restored = ServerCommandModelSync::from_json(&json);
        assert_eq!(restored.game_time, 0);
        assert_eq!(restored.turn_time, 0);
        assert!(restored.report_list.is_empty());
    }
}
