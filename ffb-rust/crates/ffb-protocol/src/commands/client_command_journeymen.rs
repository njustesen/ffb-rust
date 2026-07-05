/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandJourneymen.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandJourneymen {
    /// Java: `fSlots`
    pub slots: Vec<i32>,
    /// Java: `fPositionIds`
    pub position_ids: Vec<String>,
}

impl ClientCommandJourneymen {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `addSlot(int)`
    pub fn add_slot(&mut self, slot: i32) {
        self.slots.push(slot);
    }

    /// Java: `addPositionId(String)`
    pub fn add_position_id(&mut self, position_id: String) {
        self.position_ids.push(position_id);
    }

    /// Java: `getSlots()`
    pub fn get_slots(&self) -> &[i32] {
        &self.slots
    }

    /// Java: `getPositionIds()`
    pub fn get_position_ids(&self) -> &[String] {
        &self.position_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_slots_and_positions() {
        let mut cmd = ClientCommandJourneymen::new();
        cmd.add_slot(1);
        cmd.add_slot(2);
        cmd.add_position_id("pos_lineman".to_string());
        assert_eq!(cmd.get_slots(), &[1, 2]);
        assert_eq!(cmd.get_position_ids(), &["pos_lineman"]);
    }

    #[test]
    fn default_empty_vecs() {
        let cmd = ClientCommandJourneymen::new();
        assert!(cmd.get_slots().is_empty());
        assert!(cmd.get_position_ids().is_empty());
    }
}
