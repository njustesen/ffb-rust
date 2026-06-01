// Data loading: rosters, skills, inducements, prayers loaded from data/ JSON files

pub mod roster_json;
pub mod loader;

pub use loader::{
    bb2020_rosters, bb2016_rosters, bb2025_rosters,
    BB2020_ROSTERS_JSON, BB2016_ROSTERS_JSON, BB2025_ROSTERS_JSON,
    STAR_PLAYERS, BB2020_SKILLS, BB2016_SKILLS, BB2025_SKILLS, COMMON_SKILLS,
    BB2020_INDUCEMENTS, BB2016_INDUCEMENTS, BB2025_INDUCEMENTS,
    BB2020_PRAYERS, BB2025_PRAYERS,
};
pub use roster_json::{RosterJson, PositionJson, StarPlayerJson, StarPlayersJson};
