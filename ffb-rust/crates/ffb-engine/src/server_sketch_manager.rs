use std::collections::HashMap;

/// Manages player path sketches per WebSocket session — 1:1 translation of Java ServerSketchManager.
pub struct ServerSketchManager {
    sketches_by_session: HashMap<String, Vec<Sketch>>,
}

/// A single sketch path with coordinates and label.
#[derive(Debug, Clone)]
pub struct Sketch {
    id: String,
    coordinates: Vec<(i32, i32)>,
    label: Option<String>,
    rgb: i32,
}

impl Sketch {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), coordinates: Vec::new(), label: None, rgb: 0 }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Not a direct Java getter (Java exposes this state ad hoc where needed) — added so
    /// other crates (e.g. `ffb-server`'s `ServerCommandHandlerJoinReplay`) can read the
    /// coordinate list without reaching into a private field.
    pub fn coordinates(&self) -> &[(i32, i32)] {
        &self.coordinates
    }

    pub fn add_coordinate(&mut self, x: i32, y: i32) {
        self.coordinates.push((x, y));
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    pub fn set_rgb(&mut self, rgb: i32) {
        self.rgb = rgb;
    }
}

impl ServerSketchManager {
    pub fn new() -> Self {
        Self { sketches_by_session: HashMap::new() }
    }

    pub fn get_sketches(&mut self, session_id: &str) -> &mut Vec<Sketch> {
        self.sketches_by_session.entry(session_id.to_string()).or_default()
    }

    pub fn add_sketch(&mut self, session_id: &str, sketch: Sketch) {
        self.get_sketches(session_id).push(sketch);
    }

    pub fn add_path_coordinate(&mut self, session_id: &str, sketch_id: &str, x: i32, y: i32) {
        if let Some(s) = self.get_sketches(session_id).iter_mut().find(|s| s.id == sketch_id) {
            s.add_coordinate(x, y);
        }
    }

    pub fn set_label(&mut self, session_id: &str, sketch_id: &str, label: impl Into<String>) {
        let label = label.into();
        if let Some(s) = self.get_sketches(session_id).iter_mut().find(|s| s.id == sketch_id) {
            s.set_label(label);
        }
    }

    pub fn set_rgb(&mut self, session_id: &str, sketch_id: &str, rgb: i32) {
        if let Some(s) = self.get_sketches(session_id).iter_mut().find(|s| s.id == sketch_id) {
            s.set_rgb(rgb);
        }
    }

    pub fn remove_sketches(&mut self, session_id: &str, ids: &[&str]) {
        if let Some(sketches) = self.sketches_by_session.get_mut(session_id) {
            sketches.retain(|s| !ids.contains(&s.id.as_str()));
        }
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.sketches_by_session.remove(session_id);
    }
}

impl Default for ServerSketchManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_sketch() {
        let mut manager = ServerSketchManager::new();
        manager.add_sketch("sess1", Sketch::new("sketch1"));
        assert_eq!(manager.get_sketches("sess1").len(), 1);
    }

    #[test]
    fn test_remove_session() {
        let mut manager = ServerSketchManager::new();
        manager.add_sketch("sess1", Sketch::new("sketch1"));
        manager.remove_session("sess1");
        assert_eq!(manager.get_sketches("sess1").len(), 0);
    }
}
