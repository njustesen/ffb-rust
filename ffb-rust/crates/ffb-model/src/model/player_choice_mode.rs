use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.PlayerChoiceMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerChoiceMode {
    TENTACLES,
    SHADOWING,
    DIVING_TACKLE,
    FEED,
    DIVING_CATCH,
    DECLARE_DIVING_CATCH,
    CARD,
    BLOCK,
    MVP,
    ANIMAL_SAVAGERY,
    IRON_MAN,
    KNUCKLE_DUSTERS,
    BLESSED_STATUE_OF_NUFFLE,
    ASSIGN_TOUCHDOWN,
    BRIBERY_AND_CORRUPTION,
    INDOMITABLE,
    PICK_ME_UP,
    LORD_OF_CHAOS,
    WISDOM,
    RAIDING_PARTY,
    BALEFUL_HEX,
    BLACK_INK,
    QUICK_BITE,
    FURIOUS_OUTBURST,
    SOLID_DEFENCE,
    CHARGE,
    ARM_BAR,
    AUTO_GAZE_ZOAT,
}

impl PlayerChoiceMode {
    pub fn get_name(self) -> &'static str {
        match self {
            PlayerChoiceMode::TENTACLES => "tentacles",
            PlayerChoiceMode::SHADOWING => "shadowing",
            PlayerChoiceMode::DIVING_TACKLE => "divingTackle",
            PlayerChoiceMode::FEED => "feed",
            PlayerChoiceMode::DIVING_CATCH => "divingCatch",
            PlayerChoiceMode::DECLARE_DIVING_CATCH => "declareDivingCatch",
            PlayerChoiceMode::CARD => "card",
            PlayerChoiceMode::BLOCK => "block",
            PlayerChoiceMode::MVP => "mvp",
            PlayerChoiceMode::ANIMAL_SAVAGERY => "animalSavagery",
            PlayerChoiceMode::IRON_MAN => "ironMan",
            PlayerChoiceMode::KNUCKLE_DUSTERS => "knuckleDusters",
            PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE => "blessedStatueOfNuffle",
            PlayerChoiceMode::ASSIGN_TOUCHDOWN => "assignTouchdown",
            PlayerChoiceMode::BRIBERY_AND_CORRUPTION => "briberyAndCorruption",
            PlayerChoiceMode::INDOMITABLE => "indomitable",
            PlayerChoiceMode::PICK_ME_UP => "pickMeUp",
            PlayerChoiceMode::LORD_OF_CHAOS => "lordOfChaos",
            PlayerChoiceMode::WISDOM => "wisdomOfTheWhiteDwarf",
            PlayerChoiceMode::RAIDING_PARTY => "raidingParty",
            PlayerChoiceMode::BALEFUL_HEX => "balefulHex",
            PlayerChoiceMode::BLACK_INK => "blackInk",
            PlayerChoiceMode::QUICK_BITE => "quickBite",
            PlayerChoiceMode::FURIOUS_OUTBURST => "furiousOutburst",
            PlayerChoiceMode::SOLID_DEFENCE => "solidDefence",
            PlayerChoiceMode::CHARGE => "charge",
            PlayerChoiceMode::ARM_BAR => "armBar",
            PlayerChoiceMode::AUTO_GAZE_ZOAT => "autoGazeZoat",
        }
    }

    pub fn is_use_player_position(self) -> bool {
        match self {
            PlayerChoiceMode::CARD => false,
            PlayerChoiceMode::MVP => false,
            PlayerChoiceMode::IRON_MAN => false,
            PlayerChoiceMode::KNUCKLE_DUSTERS => false,
            PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE => false,
            PlayerChoiceMode::ASSIGN_TOUCHDOWN => false,
            PlayerChoiceMode::BRIBERY_AND_CORRUPTION => false,
            PlayerChoiceMode::PICK_ME_UP => false,
            PlayerChoiceMode::LORD_OF_CHAOS => false,
            PlayerChoiceMode::RAIDING_PARTY => false,
            PlayerChoiceMode::SOLID_DEFENCE => false,
            PlayerChoiceMode::CHARGE => false,
            _ => true,
        }
    }

    pub fn is_preselect(self) -> bool {
        match self {
            PlayerChoiceMode::PICK_ME_UP => true,
            _ => false,
        }
    }

    pub fn get_dialog_header(self, nr_of_players: i32) -> String {
        match self {
            PlayerChoiceMode::TENTACLES => "Select a player to use Tentacles".to_string(),
            PlayerChoiceMode::SHADOWING => "Select a player to use Shadowing".to_string(),
            PlayerChoiceMode::DIVING_TACKLE => "Select a player to use Diving Tackle".to_string(),
            PlayerChoiceMode::FEED => "Select a player to feed on".to_string(),
            PlayerChoiceMode::DIVING_CATCH => "Select a player to use Diving Catch".to_string(),
            PlayerChoiceMode::DECLARE_DIVING_CATCH => "Select ALL players that should try to catch the ball".to_string(),
            PlayerChoiceMode::CARD => "Select a player to play this card on".to_string(),
            PlayerChoiceMode::BLOCK => "Select a player to block".to_string(),
            PlayerChoiceMode::MVP => format!("Nominate {} for the MVP", nr_of_players),
            PlayerChoiceMode::ANIMAL_SAVAGERY => "Select a player to lash out against".to_string(),
            PlayerChoiceMode::IRON_MAN => "Select a player to become Iron Man".to_string(),
            PlayerChoiceMode::KNUCKLE_DUSTERS => "Select a player to obtain Knuckle Dusters".to_string(),
            PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE => "Select a player to receive the Blessed Statue of Nuffle".to_string(),
            PlayerChoiceMode::ASSIGN_TOUCHDOWN => "Assign a touchdown to one of your players".to_string(),
            PlayerChoiceMode::BRIBERY_AND_CORRUPTION => "Select a player to use Bribery and Corruption for".to_string(),
            PlayerChoiceMode::INDOMITABLE => "Select a player to use Indomitable against".to_string(),
            PlayerChoiceMode::PICK_ME_UP => "Select players to be picked up".to_string(),
            PlayerChoiceMode::LORD_OF_CHAOS => "Select the player of which to use Lord of Chaos".to_string(),
            PlayerChoiceMode::WISDOM => "Select the player of which to use Wisdom of the White Dwarf".to_string(),
            PlayerChoiceMode::RAIDING_PARTY => "Select the player to move".to_string(),
            PlayerChoiceMode::BALEFUL_HEX => "Select opponent to miss a turn".to_string(),
            PlayerChoiceMode::BLACK_INK => "Select opponent to lose tacklezone".to_string(),
            PlayerChoiceMode::QUICK_BITE => "Select player to take a quick bite".to_string(),
            PlayerChoiceMode::FURIOUS_OUTBURST => "Select player to attack".to_string(),
            PlayerChoiceMode::SOLID_DEFENCE => "Select players to setup again".to_string(),
            PlayerChoiceMode::CHARGE => "Select players to perform actions".to_string(),
            PlayerChoiceMode::ARM_BAR => "Select a player to use Arm Bar".to_string(),
            PlayerChoiceMode::AUTO_GAZE_ZOAT => "Select a player to Distract".to_string(),
        }
    }

    pub fn get_status_title(self) -> &'static str {
        match self {
            PlayerChoiceMode::TENTACLES => "Tentacles",
            PlayerChoiceMode::SHADOWING => "Shadowing",
            PlayerChoiceMode::DIVING_TACKLE => "Diving Tackle",
            PlayerChoiceMode::FEED => "Feed on player",
            PlayerChoiceMode::DIVING_CATCH => "Diving Catch",
            PlayerChoiceMode::DECLARE_DIVING_CATCH => "Declare Diving Catch",
            PlayerChoiceMode::CARD => "Play Card",
            PlayerChoiceMode::BLOCK => "Block",
            PlayerChoiceMode::MVP => "MVP",
            PlayerChoiceMode::ANIMAL_SAVAGERY => "Animal Savagery",
            PlayerChoiceMode::IRON_MAN => "Iron Man",
            PlayerChoiceMode::KNUCKLE_DUSTERS => "Knuckle Dusters",
            PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE => "Blessed Statue of Nuffle",
            PlayerChoiceMode::ASSIGN_TOUCHDOWN => "Touchdown from Concession",
            PlayerChoiceMode::BRIBERY_AND_CORRUPTION => "Bribery and Corruption",
            PlayerChoiceMode::INDOMITABLE => "Indomitable",
            PlayerChoiceMode::PICK_ME_UP => "Pick-me-up",
            PlayerChoiceMode::LORD_OF_CHAOS => "Lord of Chaos",
            PlayerChoiceMode::WISDOM => "Wisdom of the White Dwarf",
            PlayerChoiceMode::RAIDING_PARTY => "Raiding Party",
            PlayerChoiceMode::BALEFUL_HEX => "Baleful Hex",
            PlayerChoiceMode::BLACK_INK => "Black Ink",
            PlayerChoiceMode::QUICK_BITE => "Quick Bite",
            PlayerChoiceMode::FURIOUS_OUTBURST => "Furious Outburst",
            PlayerChoiceMode::SOLID_DEFENCE => "Solid Defence",
            PlayerChoiceMode::CHARGE => "Charge!",
            PlayerChoiceMode::ARM_BAR => "Arm Bar",
            PlayerChoiceMode::AUTO_GAZE_ZOAT => "Excuse Me, Are You a Zoat?",
        }
    }

    pub fn get_status_message(self) -> &'static str {
        match self {
            PlayerChoiceMode::TENTACLES => "Waiting for coach to use Tentacles.",
            PlayerChoiceMode::SHADOWING => "Waiting for coach to use Shadowing.",
            PlayerChoiceMode::DIVING_TACKLE => "Waiting for coach to use Diving Tackle.",
            PlayerChoiceMode::FEED => "Waiting for coach to choose player to feed on.",
            PlayerChoiceMode::DIVING_CATCH => "Waiting for coach to use Diving Catch.",
            PlayerChoiceMode::DECLARE_DIVING_CATCH => "Waiting for coach to choose all players to use Diving Catch.",
            PlayerChoiceMode::CARD => "Waiting for coach to play card on player.",
            PlayerChoiceMode::BLOCK => "Waiting for coach to choose player to block.",
            PlayerChoiceMode::MVP => "Waiting for coach to nominate players for the MVP.",
            PlayerChoiceMode::ANIMAL_SAVAGERY => "Waiting for coach to choose a player to lash out against.",
            PlayerChoiceMode::IRON_MAN => "Waiting for coach to choose a player to become Iron Man.",
            PlayerChoiceMode::KNUCKLE_DUSTERS => "Waiting for coach to choose a player to obtain Knuckle Dusters.",
            PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE => "Waiting for coach to choose a player to receive the Blessed Statue of Nuffle.",
            PlayerChoiceMode::ASSIGN_TOUCHDOWN => "Waiting for coach to assign touchdown",
            PlayerChoiceMode::BRIBERY_AND_CORRUPTION => "Waiting for coach to use Bribery and Corruption",
            PlayerChoiceMode::INDOMITABLE => "Waiting for coach to use Indomitable",
            PlayerChoiceMode::PICK_ME_UP => "Waiting for coach to choose players to be picked up",
            PlayerChoiceMode::LORD_OF_CHAOS => "Waiting for coach to select player for Lord of Chaos",
            PlayerChoiceMode::WISDOM => "Waiting for coach to select player for Wisdom of the White Dwarf",
            PlayerChoiceMode::RAIDING_PARTY => "Waiting for coach to select player to move",
            PlayerChoiceMode::BALEFUL_HEX => "Waiting for coach to select player to miss a turn",
            PlayerChoiceMode::BLACK_INK => "Waiting for coach to select player to lose tacklezone",
            PlayerChoiceMode::QUICK_BITE => "Waiting for coach to select player for a quick bite",
            PlayerChoiceMode::FURIOUS_OUTBURST => "Waiting for coach to select player to attack with furious outburst",
            PlayerChoiceMode::SOLID_DEFENCE => "Waiting for coach to select players to setup again",
            PlayerChoiceMode::CHARGE => "Waiting for coach to select players to perform actions",
            PlayerChoiceMode::ARM_BAR => "Waiting for coach to choose a player to use Arm Bar.",
            PlayerChoiceMode::AUTO_GAZE_ZOAT => "Waiting for coach to choose a player to Distract.",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name().eq_ignore_ascii_case(name))
    }

    fn all() -> &'static [PlayerChoiceMode] {
        &[
            PlayerChoiceMode::TENTACLES,
            PlayerChoiceMode::SHADOWING,
            PlayerChoiceMode::DIVING_TACKLE,
            PlayerChoiceMode::FEED,
            PlayerChoiceMode::DIVING_CATCH,
            PlayerChoiceMode::DECLARE_DIVING_CATCH,
            PlayerChoiceMode::CARD,
            PlayerChoiceMode::BLOCK,
            PlayerChoiceMode::MVP,
            PlayerChoiceMode::ANIMAL_SAVAGERY,
            PlayerChoiceMode::IRON_MAN,
            PlayerChoiceMode::KNUCKLE_DUSTERS,
            PlayerChoiceMode::BLESSED_STATUE_OF_NUFFLE,
            PlayerChoiceMode::ASSIGN_TOUCHDOWN,
            PlayerChoiceMode::BRIBERY_AND_CORRUPTION,
            PlayerChoiceMode::INDOMITABLE,
            PlayerChoiceMode::PICK_ME_UP,
            PlayerChoiceMode::LORD_OF_CHAOS,
            PlayerChoiceMode::WISDOM,
            PlayerChoiceMode::RAIDING_PARTY,
            PlayerChoiceMode::BALEFUL_HEX,
            PlayerChoiceMode::BLACK_INK,
            PlayerChoiceMode::QUICK_BITE,
            PlayerChoiceMode::FURIOUS_OUTBURST,
            PlayerChoiceMode::SOLID_DEFENCE,
            PlayerChoiceMode::CHARGE,
            PlayerChoiceMode::ARM_BAR,
            PlayerChoiceMode::AUTO_GAZE_ZOAT,
        ]
    }
}
