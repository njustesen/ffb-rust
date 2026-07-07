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

    #[test]
    fn block_kind_names() {
        assert_eq!(BlockKind::Normal.name(), "normal");
        assert_eq!(BlockKind::Blitz.name(), "blitz");
        assert_eq!(BlockKind::Foul.name(), "foul");
    }

    #[test]
    fn dice_category_kind_names() {
        assert_eq!(DiceCategoryKind::Block.name(), "Block");
        assert_eq!(DiceCategoryKind::D6.name(), "D6");
        assert_eq!(DiceCategoryKind::D8.name(), "D8");
        assert_eq!(DiceCategoryKind::Scatter.name(), "Scatter");
        assert_eq!(DiceCategoryKind::ThrowIn.name(), "ThrowIn");
        assert_eq!(DiceCategoryKind::Direction.name(), "Direction");
        assert_eq!(DiceCategoryKind::ArtiZan.name(), "ArtiZan");
    }

    #[test]
    fn dice_decoration_fields() {
        let d = DiceDecoration {
            coordinate_x: 3,
            coordinate_y: 7,
            nr_of_dice: 2,
            block_kind: BlockKind::Normal,
        };
        assert_eq!(d.coordinate_x, 3);
        assert_eq!(d.coordinate_y, 7);
        assert_eq!(d.nr_of_dice, 2);
        assert_eq!(d.block_kind, BlockKind::Normal);
    }

    #[test]
    fn block_kind_distinct_variants() {
        assert_ne!(BlockKind::Normal, BlockKind::Blitz);
        assert_ne!(BlockKind::Foul, BlockKind::Normal);
    }

    #[test]
    fn dice_category_kind_serde_round_trip() {
        let k = DiceCategoryKind::D6;
        let json = serde_json::to_string(&k).unwrap();
        let back: DiceCategoryKind = serde_json::from_str(&json).unwrap();
        assert_eq!(k, back);
    }
}
