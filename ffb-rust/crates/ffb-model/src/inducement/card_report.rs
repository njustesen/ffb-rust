/// A roll result with description for a card effect — 1:1 translation of Java CardReport.
pub struct CardReport {
    roll: String,
    description: String,
}

impl CardReport {
    pub fn new(roll: impl Into<String>, description: impl Into<String>) -> Self {
        Self { roll: roll.into(), description: description.into() }
    }

    pub fn get_roll(&self) -> &str {
        &self.roll
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }
}
