
/// BB2025 Dodge skill behaviour. Mirrors Java
/// `com.fumbbl.ffb.server.skillbehaviour.bb2025.DodgeBehaviour`, which just calls
/// `super(1, false)` on `AbstractDodgingBehaviour` with no BB2025-specific override.
///
/// The real `StepModifierTrait` logic (dodge-choice default, `ReportSkillUse`) is
/// `AbstractDodgingStepModifier`, registered directly by
/// `registry.rs::build_bb2025` as `AbstractDodgingBehaviour::register_into(&mut reg,
/// SkillId::Dodge, 1, false)` — see `skill_behaviour/mixed/abstract_dodging_behaviour.rs`.
/// This type is an intentionally inert marker (matches the BB2016 `DodgeBehaviour`
/// precedent of not double-registering already-real logic). The previous doc comment
/// here ("No StepModifier") was itself stale/incorrect — Java's `AbstractDodgingBehaviour`
/// superclass does register one; only the constructor call was BB2025-trivial.
pub struct DodgeBehaviour;

impl DodgeBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for DodgeBehaviour {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_game() -> ffb_model::model::game::Game {
        let home = ffb_model::model::team::Team {
            id: "home".into(), name: "Home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = home.clone();
        ffb_model::model::game::Game::new(home, away, ffb_model::enums::Rules::Bb2025)
    }

}
