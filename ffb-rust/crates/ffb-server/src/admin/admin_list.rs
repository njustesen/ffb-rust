/// 1:1 translation of com.fumbbl.ffb.server.admin.AdminList.
use super::admin_list_entry::AdminListEntry;

pub struct AdminList {
    entries: Vec<AdminListEntry>,
}

impl AdminList {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(&mut self, entry: AdminListEntry) {
        self.entries.push(entry);
    }

    pub fn get_entries(&self) -> &[AdminListEntry] {
        &self.entries
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }
}

impl Default for AdminList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let list = AdminList::new();
        assert_eq!(list.size(), 0);
    }
}
