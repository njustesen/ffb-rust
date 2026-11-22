use serde::{Deserialize, Serialize};
use crate::types::FieldCoordinate;
use super::block_kind::BlockKind;

/// 1:1 translation of com.fumbbl.ffb.DiceDecoration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiceDecoration {
    pub coordinate: Option<FieldCoordinate>,
    pub nr_of_dice: i32,
    pub block_kind: Option<BlockKind>,
}

impl DiceDecoration {
    pub fn new(coordinate: FieldCoordinate, nr_of_dice: i32, block_kind: BlockKind) -> Self {
        DiceDecoration { coordinate: Some(coordinate), nr_of_dice, block_kind: Some(block_kind) }
    }

    pub fn get_coordinate(&self) -> Option<&FieldCoordinate> { self.coordinate.as_ref() }
    pub fn get_nr_of_dice(&self) -> i32 { self.nr_of_dice }
    pub fn get_block_kind(&self) -> Option<BlockKind> { self.block_kind }

    /// Java: `DiceDecoration.toJsonValue()` (`blockKind` only when non-null).
    pub fn to_json_value(&self) -> serde_json::Value {
        use crate::util::java_json as j;
        let mut o = j::JsonObject::new();
        o.insert("coordinate".into(), j::field_coordinate_or_null(self.coordinate));
        o.insert("nrOfDice".into(), serde_json::Value::from(self.nr_of_dice));
        if let Some(kind) = self.block_kind {
            o.insert("blockKind".into(), serde_json::Value::from(block_kind_name(kind)));
        }
        serde_json::Value::Object(o)
    }

    /// Java: `new DiceDecoration().initFrom(source, jsonValue)`; `BlockKind.valueOf` throws on an
    /// unknown or null name (`Err`).
    pub fn init_from(json: &serde_json::Value) -> Result<Self, String> {
        use crate::util::java_json as j;
        let o = j::to_json_object(json)?;
        let mut d = DiceDecoration {
            coordinate: j::get_field_coordinate(o, "coordinate")?,
            nr_of_dice: j::get_int(o, "nrOfDice")?,
            block_kind: None,
        };
        if j::is_defined_in(o, "blockKind") {
            let name = j::get_string(o, "blockKind")?;
            d.block_kind = Some(
                name.as_deref()
                    .and_then(block_kind_value_of)
                    .ok_or_else(|| format!("DiceDecoration: BlockKind.valueOf({name:?}) throws"))?,
            );
        }
        Ok(d)
    }

    pub fn transform(&self) -> Self {
        DiceDecoration {
            coordinate: self.coordinate.as_ref().map(|c| c.transform()),
            nr_of_dice: self.nr_of_dice,
            block_kind: self.block_kind,
        }
    }
}

/// Java `BlockKind.name()`.
fn block_kind_name(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::BLOCK => "BLOCK",
        BlockKind::STAB => "STAB",
        BlockKind::VOMIT => "VOMIT",
        BlockKind::CHAINSAW => "CHAINSAW",
    }
}

/// Java `BlockKind.valueOf(name)` (exact constant name; `None` where Java throws).
fn block_kind_value_of(name: &str) -> Option<BlockKind> {
    [BlockKind::BLOCK, BlockKind::STAB, BlockKind::VOMIT, BlockKind::CHAINSAW]
        .into_iter()
        .find(|k| block_kind_name(*k) == name)
}

impl Default for DiceDecoration {
    fn default() -> Self {
        DiceDecoration { coordinate: None, nr_of_dice: 0, block_kind: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FieldCoordinate;

    #[test]
    fn transform_preserves_dice_count_and_block_kind() {
        let d = DiceDecoration::new(FieldCoordinate::new(5, 3), 3, BlockKind::STAB);
        let t = d.transform();
        assert_eq!(t.get_nr_of_dice(), 3);
        assert_eq!(t.get_block_kind(), Some(BlockKind::STAB));
    }

    #[test]
    fn transform_mirrors_coordinate() {
        let coord = FieldCoordinate::new(3, 5);
        let d = DiceDecoration::new(coord, 1, BlockKind::BLOCK);
        let t = d.transform();
        assert_eq!(t.get_coordinate(), Some(&coord.transform()));
        assert_ne!(t.get_coordinate(), Some(&coord));
    }

    #[test]
    fn transform_of_default_keeps_none_coordinate() {
        let d = DiceDecoration::default();
        let t = d.transform();
        assert_eq!(t.get_coordinate(), None);
        assert_eq!(t.get_nr_of_dice(), 0);
        assert_eq!(t.get_block_kind(), None);
    }
}
