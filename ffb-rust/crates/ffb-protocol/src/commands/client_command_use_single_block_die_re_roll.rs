use ffb_model::enums::ReRollSource;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseSingleBlockDieReRoll`.
/// Sent to re-roll a single block die result.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseSingleBlockDieReRoll {
    /// Java: `dieIndex`
    pub die_index: i32,
    /// Java: `reRollSource`
    pub re_roll_source: Option<ReRollSource>,
}

impl ClientCommandUseSingleBlockDieReRoll {
    pub fn new(die_index: i32) -> Self { Self { die_index, re_roll_source: None } }
    pub fn get_die_index(&self) -> i32 { self.die_index }
    pub fn get_re_roll_source(&self) -> Option<&ReRollSource> { self.re_roll_source.as_ref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn die_index_stored() {
        assert_eq!(ClientCommandUseSingleBlockDieReRoll::new(1).get_die_index(), 1);
    }
    #[test]
    fn default_no_source() {
        assert!(ClientCommandUseSingleBlockDieReRoll::new(0).re_roll_source.is_none());
    }
}
