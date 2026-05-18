pub mod block;
pub mod inducements;
pub mod injury;
pub mod movement;
pub mod pass;
pub mod special_rules;

pub use block::{block_dice_count, BlockOutcome};
pub use crate::types::InducementState;
pub use inducements::{wizard_lightning_bolt, wizard_fireball, use_bribe, babes_ko_recovery};
pub use injury::{armor_roll, injury_roll, casualty_roll, apply_regeneration, apply_apothecary, ArmorOutcome, InjuryResolution};
pub use movement::{MoveOutcome, execute_move_step};
pub use pass::{pass_min_roll, pass_scatter_coord, PassResult};
