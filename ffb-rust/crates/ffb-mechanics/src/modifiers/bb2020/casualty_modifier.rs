pub struct CasualtyModifier {
    pub name: String,
    pub modifier: i32,
    applies_to_context: Option<Box<dyn Fn(&ffb_model::model::Player) -> bool + Send + Sync>>,
}

impl CasualtyModifier {
    pub fn new(name: impl Into<String>, modifier: i32) -> Self {
        Self { name: name.into(), modifier, applies_to_context: None }
    }

    pub fn with_predicate(mut self, f: impl Fn(&ffb_model::model::Player) -> bool + Send + Sync + 'static) -> Self {
        self.applies_to_context = Some(Box::new(f));
        self
    }

    pub fn get_modifier(&self) -> i32 { self.modifier }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn applies_to_context(&self, player: &ffb_model::model::Player) -> bool {
        self.applies_to_context.as_ref().map(|f| f(player)).unwrap_or(true)
    }
    pub fn report_string(&self) -> String { format!("{} {}", self.modifier, self.name) }
}
