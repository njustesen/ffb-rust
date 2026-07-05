use ffb_model::model::sketch::sketch::Sketch;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAddSketches`.
/// Sends one or more sketches from a coach to all clients.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAddSketches {
    /// Java: `coach` — the coach who drew the sketches.
    pub coach: String,
    /// Java: `sketches` — the list of sketch objects.
    pub sketches: Vec<Sketch>,
}

impl ServerCommandAddSketches {
    pub fn new(coach: impl Into<String>, sketches: Vec<Sketch>) -> Self {
        Self { coach: coach.into(), sketches }
    }
    pub fn get_coach(&self) -> &str { &self.coach }
    pub fn get_sketches(&self) -> &[Sketch] { &self.sketches }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandAddSketches::new("Alice", vec![Sketch::new()]);
        assert_eq!(cmd.get_coach(), "Alice");
        assert_eq!(cmd.get_sketches().len(), 1);
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandAddSketches::default();
        assert!(cmd.coach.is_empty());
        assert!(cmd.sketches.is_empty());
    }
}
