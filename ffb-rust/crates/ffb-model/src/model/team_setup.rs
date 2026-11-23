use std::any::Any;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::enums::{PlayerState, PS_RESERVE, PS_STANDING};
use crate::model::game::Game;
use crate::types::FieldCoordinate;
use crate::util::util_box::UtilBox;
use crate::xml::util_xml::{get_int_attribute, get_string_attribute, XmlAttributes};
use crate::xml::IXmlReadable;

/// 1:1 translation of com.fumbbl.ffb.TeamSetup.
///
/// Java keeps `fCoordinateByPlayerNr` in a `HashMap<Integer, ...>`; for the small player numbers
/// a setup holds, `keySet()` iterates in ascending order, which the `BTreeMap` reproduces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSetup {
    /// Java: `fName`
    pub name: String,
    /// Java: `fTeamId`
    pub team_id: String,
    /// Java: `fCoordinateByPlayerNr`
    pub coordinate_by_player_nr: BTreeMap<i32, FieldCoordinate>,
    /// Java: transient `fCurrentPlayerNr` (XML parsing).
    #[serde(skip)]
    current_player_nr: i32,
}

impl TeamSetup {
    pub const XML_TAG: &'static str = "teamSetup";

    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getPlayerNumbers()`.
    pub fn get_player_numbers(&self) -> Vec<i32> {
        self.coordinate_by_player_nr.keys().copied().collect()
    }

    /// Java: `getCoordinates()`.
    pub fn get_coordinates(&self) -> Vec<FieldCoordinate> {
        self.coordinate_by_player_nr.values().copied().collect()
    }

    /// Java: `addCoordinate(FieldCoordinate, int)`.
    pub fn add_coordinate(&mut self, coordinate: FieldCoordinate, player_nr: i32) {
        self.coordinate_by_player_nr.insert(player_nr, coordinate);
    }

    /// Java: `getCoordinate(int)`.
    pub fn get_coordinate(&self, player_nr: i32) -> Option<FieldCoordinate> {
        self.coordinate_by_player_nr.get(&player_nr).copied()
    }

    /// Java: `applyTo(Game)`: every player that can be set up next drive goes to its setup square
    /// (STANDING, active; mirrored for the away team) or, without one, to the reserve box.
    pub fn apply_to(&self, game: &mut Game) {
        let home_setup = self.team_id == game.team_home.id;
        let players: Vec<(String, i32)> = {
            let team = if home_setup { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| (p.id.clone(), p.nr)).collect()
        };
        for (player_id, nr) in players {
            let player_coordinate = self.get_coordinate(nr);
            let player_state = game.field_model.player_state(&player_id).unwrap_or(PlayerState(0));
            if player_state.can_be_set_up_next_drive() {
                if let Some(coordinate) = player_coordinate {
                    game.field_model
                        .set_player_state(&player_id, player_state.change_base(PS_STANDING).change_active(true));
                    let coordinate = if home_setup { coordinate } else { coordinate.transform() };
                    game.field_model.set_player_coordinate(&player_id, coordinate);
                } else {
                    game.field_model.set_player_state(&player_id, player_state.change_base(PS_RESERVE));
                    UtilBox::put_player_into_box(game, &player_id);
                }
            }
        }
    }
}

impl IXmlReadable for TeamSetup {
    fn start_xml_element(&mut self, _game: Option<&Game>, tag: &str, atts: &XmlAttributes) -> Option<Box<dyn IXmlReadable>> {
        if tag == Self::XML_TAG {
            self.name = get_string_attribute(atts, "name").unwrap_or_default();
            self.team_id = get_string_attribute(atts, "teamId").unwrap_or_default();
        }
        if tag == "player" {
            self.current_player_nr = get_int_attribute(atts, "nr");
        }
        if tag == "coordinate" {
            let x = get_int_attribute(atts, "x");
            let y = get_int_attribute(atts, "y");
            let nr = self.current_player_nr;
            self.add_coordinate(FieldCoordinate::new(x, y), nr);
        }
        None
    }

    fn end_xml_element(&mut self, _game: Option<&Game>, tag: &str, _value: &str) -> bool {
        tag == Self::XML_TAG
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::XmlHandler;

    #[test]
    fn parses_java_setup_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<teamSetup teamId="teamHumanKalimar" name="Human Team Setup">
  <player nr="2"><coordinate x="12" y="6" /></player>
  <player nr="1"><coordinate x="12" y="8" /></player>
</teamSetup>"#;
        let parsed = XmlHandler::parse(None, xml, Box::new(TeamSetup::new()));
        let setup = parsed.into_any().downcast::<TeamSetup>().unwrap();
        assert_eq!(setup.team_id, "teamHumanKalimar");
        assert_eq!(setup.name, "Human Team Setup");
        assert_eq!(setup.get_player_numbers(), vec![1, 2]);
        assert_eq!(setup.get_coordinate(2), Some(FieldCoordinate::new(12, 6)));
    }
}
