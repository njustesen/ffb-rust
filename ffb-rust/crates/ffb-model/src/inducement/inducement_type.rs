/// 1:1 translation of Java InducementType.
pub struct InducementType {
    name: String,
    description: String,
    singular: String,
    plural: String,
    uses_generic_slot: bool,
    priority: i32,
}

impl InducementType {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        singular: impl Into<String>,
        plural: impl Into<String>,
        uses_generic_slot: bool,
        priority: i32,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            singular: singular.into(),
            plural: plural.into(),
            uses_generic_slot,
            priority,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_singular(&self) -> &str {
        &self.singular
    }

    pub fn get_plural(&self) -> &str {
        &self.plural
    }

    pub fn uses_generic_slot(&self) -> bool {
        self.uses_generic_slot
    }

    pub fn get_priority(&self) -> i32 {
        self.priority
    }
}
