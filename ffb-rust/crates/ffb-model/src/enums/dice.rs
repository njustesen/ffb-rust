use serde::{Deserialize, Serialize};

/// Which category of die was rolled (for test harness purposes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiceCategoryKind {
    Block,
    D6,
    D8,
    Scatter,
    ThrowIn,
    Direction,
    ArtiZan,
}

impl DiceCategoryKind {
    pub fn name(self) -> &'static str {
        match self {
            DiceCategoryKind::Block => "Block",
            DiceCategoryKind::D6 => "D6",
            DiceCategoryKind::D8 => "D8",
            DiceCategoryKind::Scatter => "Scatter",
            DiceCategoryKind::ThrowIn => "ThrowIn",
            DiceCategoryKind::Direction => "Direction",
            DiceCategoryKind::ArtiZan => "ArtiZan",
        }
    }
}

/// Decoration shown on the field for pending block dice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceDecoration {
    pub coordinate_x: i8,
    pub coordinate_y: i8,
    pub nr_of_dice: i32,
    /// Distinguishes block / foul / special block kinds.
    pub block_kind: BlockKind,
}

/// The kind of block action generating the dice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockKind {
    Normal,
    Blitz,
    Foul,
}

impl BlockKind {
    pub fn name(self) -> &'static str {
        match self {
            BlockKind::Normal => "normal",
            BlockKind::Blitz => "blitz",
            BlockKind::Foul => "foul",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_kind_serde() {
        let k = BlockKind::Blitz;
        let json = serde_json::to_string(&k).unwrap();
        let back: BlockKind = serde_json::from_str(&json).unwrap();
        assert_eq!(k, back);
    }
}
