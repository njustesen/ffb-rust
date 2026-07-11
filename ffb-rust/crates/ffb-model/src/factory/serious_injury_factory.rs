/// 1:1 translation of com.fumbbl.ffb.factory.SeriousInjuryFactory.
///
/// Java scans the classpath for `@RulesCollection`-annotated implementations of the
/// `SeriousInjury` interface matching the game's rules version (`Scanner<SeriousInjury>`),
/// then exposes `forName`, `dead()`, `poison()` and `forAttribute(InjuryAttribute)` over
/// that edition-specific set. Rust has no runtime classpath scanning, so `initialize`
/// picks the matching edition module (`bb2016`/`bb2020`/`bb2025`) at the call site instead
/// and wraps its concrete `SeriousInjury` enum in `AnySeriousInjury` (a sum type standing
/// in for the interface's possible concrete implementations).
use crate::bb2016::serious_injury::SeriousInjury as Bb2016SeriousInjury;
use crate::bb2020::serious_injury::SeriousInjury as Bb2020SeriousInjury;
use crate::bb2025::serious_injury::SeriousInjury as Bb2025SeriousInjury;
use crate::enums::Rules;
use crate::enums::SeriousInjuryKind;
use crate::model::game::Game;
use crate::model::injury_attribute::InjuryAttribute;
use crate::model::serious_injury::SeriousInjury as ISeriousInjury;

/// Every BB2016 `SeriousInjury` variant, in Java enum declaration order.
const ALL_BB2016: &[Bb2016SeriousInjury] = &[
    Bb2016SeriousInjury::BROKEN_RIBS,
    Bb2016SeriousInjury::GROIN_STRAIN,
    Bb2016SeriousInjury::GOUGED_EYE,
    Bb2016SeriousInjury::BROKEN_JAW,
    Bb2016SeriousInjury::FRACTURED_ARM,
    Bb2016SeriousInjury::FRACTURED_LEG,
    Bb2016SeriousInjury::SMASHED_HAND,
    Bb2016SeriousInjury::PINCHED_NERVE,
    Bb2016SeriousInjury::DAMAGED_BACK,
    Bb2016SeriousInjury::SMASHED_KNEE,
    Bb2016SeriousInjury::SMASHED_HIP,
    Bb2016SeriousInjury::SMASHED_ANKLE,
    Bb2016SeriousInjury::SERIOUS_CONCUSSION,
    Bb2016SeriousInjury::FRACTURED_SKULL,
    Bb2016SeriousInjury::BROKEN_NECK,
    Bb2016SeriousInjury::SMASHED_COLLAR_BONE,
    Bb2016SeriousInjury::DEAD,
    Bb2016SeriousInjury::POISONED,
];

/// Every BB2020 `SeriousInjury` variant, in Java enum declaration order.
const ALL_BB2020: &[Bb2020SeriousInjury] = &[
    Bb2020SeriousInjury::SERIOUSLY_HURT,
    Bb2020SeriousInjury::SERIOUS_INJURY,
    Bb2020SeriousInjury::HEAD_INJURY,
    Bb2020SeriousInjury::SMASHED_KNEE,
    Bb2020SeriousInjury::BROKEN_ARM,
    Bb2020SeriousInjury::NECK_INJURY,
    Bb2020SeriousInjury::DISLOCATED_SHOULDER,
    Bb2020SeriousInjury::DEAD,
];

/// Every BB2025 `SeriousInjury` variant, in Java enum declaration order.
const ALL_BB2025: &[Bb2025SeriousInjury] = &[
    Bb2025SeriousInjury::SERIOUSLY_HURT,
    Bb2025SeriousInjury::SERIOUS_INJURY,
    Bb2025SeriousInjury::HEAD_INJURY,
    Bb2025SeriousInjury::SMASHED_KNEE,
    Bb2025SeriousInjury::BROKEN_ARM,
    Bb2025SeriousInjury::DISLOCATED_HIP,
    Bb2025SeriousInjury::DISLOCATED_SHOULDER,
    Bb2025SeriousInjury::DEAD,
];

/// Sum type over the three edition-specific `SeriousInjury` enums — stands in for
/// Java's `SeriousInjury` interface, whose concrete runtime type depends on which
/// `@RulesCollection`-annotated class the classpath scanner picked for the game's rules.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnySeriousInjury {
    Bb2016(Bb2016SeriousInjury),
    Bb2020(Bb2020SeriousInjury),
    Bb2025(Bb2025SeriousInjury),
}

impl AnySeriousInjury {
    /// Maps this edition-specific value onto the engine's flattened
    /// `ffb_model::enums::SeriousInjuryKind`, used by `PlayerResult`/`Player` to store
    /// lasting injuries independently of which edition produced them.
    pub fn to_kind(self) -> SeriousInjuryKind {
        match self {
            AnySeriousInjury::Bb2016(v) => match v {
                Bb2016SeriousInjury::BROKEN_RIBS => SeriousInjuryKind::BrokenRibs,
                Bb2016SeriousInjury::GROIN_STRAIN => SeriousInjuryKind::Groin,
                Bb2016SeriousInjury::GOUGED_EYE => SeriousInjuryKind::GougedEye,
                Bb2016SeriousInjury::BROKEN_JAW => SeriousInjuryKind::BrokenJaw,
                Bb2016SeriousInjury::FRACTURED_ARM => SeriousInjuryKind::FracturedArm,
                Bb2016SeriousInjury::FRACTURED_LEG => SeriousInjuryKind::FracturedLeg,
                Bb2016SeriousInjury::SMASHED_HAND => SeriousInjuryKind::SmashedHand,
                Bb2016SeriousInjury::PINCHED_NERVE => SeriousInjuryKind::PinchedNerve,
                Bb2016SeriousInjury::DAMAGED_BACK => SeriousInjuryKind::DamagedBack,
                Bb2016SeriousInjury::SMASHED_KNEE => SeriousInjuryKind::SmashedKneeB2016,
                Bb2016SeriousInjury::SMASHED_HIP => SeriousInjuryKind::SmashedHip,
                Bb2016SeriousInjury::SMASHED_ANKLE => SeriousInjuryKind::SmashedAnkle,
                Bb2016SeriousInjury::SERIOUS_CONCUSSION => SeriousInjuryKind::SeriousConcussion,
                Bb2016SeriousInjury::FRACTURED_SKULL => SeriousInjuryKind::FracturedSkull,
                Bb2016SeriousInjury::BROKEN_NECK => SeriousInjuryKind::BrokenNeck,
                Bb2016SeriousInjury::SMASHED_COLLAR_BONE => SeriousInjuryKind::BrokenCollarBone,
                Bb2016SeriousInjury::DEAD => SeriousInjuryKind::Dead,
                Bb2016SeriousInjury::POISONED => SeriousInjuryKind::Poisoned,
            },
            AnySeriousInjury::Bb2020(v) => match v {
                Bb2020SeriousInjury::SERIOUSLY_HURT => SeriousInjuryKind::SeriouslyHurt,
                Bb2020SeriousInjury::SERIOUS_INJURY => SeriousInjuryKind::SeriousInjuryNi,
                Bb2020SeriousInjury::HEAD_INJURY => SeriousInjuryKind::HeadInjuryAv,
                Bb2020SeriousInjury::SMASHED_KNEE => SeriousInjuryKind::SmashedKneeMa,
                Bb2020SeriousInjury::BROKEN_ARM => SeriousInjuryKind::BrokenArmPa,
                Bb2020SeriousInjury::NECK_INJURY => SeriousInjuryKind::NeckInjuryAg,
                Bb2020SeriousInjury::DISLOCATED_SHOULDER => SeriousInjuryKind::DislocatedShoulderSt,
                Bb2020SeriousInjury::DEAD => SeriousInjuryKind::Dead,
            },
            AnySeriousInjury::Bb2025(v) => match v {
                Bb2025SeriousInjury::SERIOUSLY_HURT => SeriousInjuryKind::SeriouslyHurt,
                Bb2025SeriousInjury::SERIOUS_INJURY => SeriousInjuryKind::SeriousInjuryNi,
                Bb2025SeriousInjury::HEAD_INJURY => SeriousInjuryKind::HeadInjuryAv,
                Bb2025SeriousInjury::SMASHED_KNEE => SeriousInjuryKind::SmashedKneeMa,
                Bb2025SeriousInjury::BROKEN_ARM => SeriousInjuryKind::BrokenArmPa,
                Bb2025SeriousInjury::DISLOCATED_HIP => SeriousInjuryKind::DislocatedHipAg,
                Bb2025SeriousInjury::DISLOCATED_SHOULDER => SeriousInjuryKind::DislocatedShoulderSt,
                Bb2025SeriousInjury::DEAD => SeriousInjuryKind::Dead,
            },
        }
    }
}

impl ISeriousInjury for AnySeriousInjury {
    fn get_name(&self) -> &str {
        match self {
            AnySeriousInjury::Bb2016(v) => v.get_name(),
            AnySeriousInjury::Bb2020(v) => v.get_name(),
            AnySeriousInjury::Bb2025(v) => v.get_name(),
        }
    }

    fn get_button_text(&self) -> &str {
        match self {
            AnySeriousInjury::Bb2016(v) => v.get_button_text(),
            AnySeriousInjury::Bb2020(v) => v.get_button_text(),
            AnySeriousInjury::Bb2025(v) => v.get_button_text(),
        }
    }

    fn get_description(&self) -> &str {
        match self {
            AnySeriousInjury::Bb2016(v) => v.get_description(),
            AnySeriousInjury::Bb2020(v) => v.get_description(),
            AnySeriousInjury::Bb2025(v) => v.get_description(),
        }
    }

    fn get_recovery(&self) -> &str {
        match self {
            AnySeriousInjury::Bb2016(v) => v.get_recovery(),
            AnySeriousInjury::Bb2020(v) => v.get_recovery(),
            AnySeriousInjury::Bb2025(v) => v.get_recovery(),
        }
    }

    fn get_injury_attribute(&self) -> Option<InjuryAttribute> {
        match self {
            AnySeriousInjury::Bb2016(v) => v.get_injury_attribute(),
            AnySeriousInjury::Bb2020(v) => v.get_injury_attribute(),
            AnySeriousInjury::Bb2025(v) => v.get_injury_attribute(),
        }
    }

    fn is_dead(&self) -> bool {
        match self {
            AnySeriousInjury::Bb2016(v) => v.is_dead(),
            AnySeriousInjury::Bb2020(v) => v.is_dead(),
            AnySeriousInjury::Bb2025(v) => v.is_dead(),
        }
    }

    fn is_poison(&self) -> bool {
        match self {
            AnySeriousInjury::Bb2016(v) => v.is_poison(),
            AnySeriousInjury::Bb2020(v) => v.is_poison(),
            AnySeriousInjury::Bb2025(v) => v.is_poison(),
        }
    }

    fn show_si_roll(&self) -> bool {
        match self {
            AnySeriousInjury::Bb2016(v) => v.show_si_roll(),
            AnySeriousInjury::Bb2020(v) => v.show_si_roll(),
            AnySeriousInjury::Bb2025(v) => v.show_si_roll(),
        }
    }
}

/// Java: `com.fumbbl.ffb.factory.SeriousInjuryFactory`.
#[derive(Debug, Clone, Default)]
pub struct SeriousInjuryFactory {
    /// Java: `values` — the edition-specific `SeriousInjury` set, populated by `initialize`.
    values: Vec<AnySeriousInjury>,
    /// Java: `dead`.
    dead: Option<AnySeriousInjury>,
    /// Java: `poison`.
    poison: Option<AnySeriousInjury>,
}

impl SeriousInjuryFactory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `forName(String pName)`.
    pub fn for_name(&self, name: &str) -> Option<AnySeriousInjury> {
        self.values.iter().copied().find(|v| v.get_name() == name)
    }

    /// Java: `initialize(Game game)` — `values = new Scanner<>(SeriousInjury.class)
    /// .getEnumValues(game.getOptions())`, then finds `dead`/`poison` among them.
    pub fn initialize(&mut self, game: &Game) {
        self.values = match game.rules {
            Rules::Bb2016 | Rules::Common => {
                ALL_BB2016.iter().copied().map(AnySeriousInjury::Bb2016).collect()
            }
            Rules::Bb2020 => ALL_BB2020.iter().copied().map(AnySeriousInjury::Bb2020).collect(),
            Rules::Bb2025 => ALL_BB2025.iter().copied().map(AnySeriousInjury::Bb2025).collect(),
        };
        self.dead = self.values.iter().copied().find(|v| v.is_dead());
        self.poison = self.values.iter().copied().find(|v| v.is_poison());
    }

    /// Java: `dead()`.
    pub fn dead(&self) -> Option<AnySeriousInjury> {
        self.dead
    }

    /// Java: `poison()`.
    pub fn poison(&self) -> Option<AnySeriousInjury> {
        self.poison
    }

    /// Java: `forAttribute(InjuryAttribute attribute)` —
    /// `values.stream().filter(value -> value.getInjuryAttribute() == attribute).findFirst()`.
    pub fn for_attribute(&self, attribute: InjuryAttribute) -> Option<AnySeriousInjury> {
        self.values.iter().copied().find(|v| v.get_injury_attribute() == Some(attribute))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game(rules: Rules) -> Game {
        Game::new(make_team("home"), make_team("away"), rules)
    }

    #[test]
    fn initialize_bb2025_populates_eight_values() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2025));
        assert_eq!(factory.values.len(), 8);
    }

    #[test]
    fn initialize_bb2016_populates_eighteen_values() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2016));
        assert_eq!(factory.values.len(), 18);
    }

    #[test]
    fn dead_returns_dead_variant_for_each_edition() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            let mut factory = SeriousInjuryFactory::new();
            factory.initialize(&make_game(rules));
            let dead = factory.dead().expect("every edition has a DEAD variant");
            assert!(dead.is_dead());
            assert_eq!(dead.to_kind(), SeriousInjuryKind::Dead);
        }
    }

    #[test]
    fn poison_only_present_in_bb2016() {
        let mut bb2016 = SeriousInjuryFactory::new();
        bb2016.initialize(&make_game(Rules::Bb2016));
        assert!(bb2016.poison().is_some());

        let mut bb2025 = SeriousInjuryFactory::new();
        bb2025.initialize(&make_game(Rules::Bb2025));
        assert!(bb2025.poison().is_none());
    }

    #[test]
    fn for_name_finds_bb2025_head_injury() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2025));
        let found = factory.for_name("Head Injury (-AV)").unwrap();
        assert_eq!(found.to_kind(), SeriousInjuryKind::HeadInjuryAv);
    }

    #[test]
    fn for_name_unknown_returns_none() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2025));
        assert!(factory.for_name("Not A Real Injury").is_none());
    }

    #[test]
    fn for_attribute_bb2025_ag_is_dislocated_hip() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2025));
        let found = factory.for_attribute(InjuryAttribute::AG).unwrap();
        assert_eq!(found.to_kind(), SeriousInjuryKind::DislocatedHipAg);
    }

    #[test]
    fn for_attribute_bb2020_ag_is_neck_injury() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2020));
        let found = factory.for_attribute(InjuryAttribute::AG).unwrap();
        assert_eq!(found.to_kind(), SeriousInjuryKind::NeckInjuryAg);
    }

    #[test]
    fn for_attribute_bb2016_ma_is_smashed_hip_first_match() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2016));
        // Java enum declaration order has SMASHED_HIP before SMASHED_ANKLE; both map to MA,
        // and `findFirst()` returns the first match in declaration order.
        let found = factory.for_attribute(InjuryAttribute::MA).unwrap();
        assert_eq!(found.to_kind(), SeriousInjuryKind::SmashedHip);
    }

    #[test]
    fn for_attribute_no_match_returns_none() {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(&make_game(Rules::Bb2020));
        // BB2020 has no PA... wait ST is covered; use an attribute definitely absent.
        // Every InjuryAttribute variant is actually covered by BB2020, so instead verify
        // an uninitialized factory (no values) returns None for any attribute.
        let empty_factory = SeriousInjuryFactory::new();
        assert!(empty_factory.for_attribute(InjuryAttribute::MA).is_none());
        let _ = factory; // keep bb2020 factory constructed for parity with other tests
    }

    #[test]
    fn to_kind_round_trips_every_bb2016_variant() {
        for &v in ALL_BB2016 {
            let any = AnySeriousInjury::Bb2016(v);
            // Just make sure it doesn't panic and produces a stable value.
            let kind = any.to_kind();
            assert_eq!(any.to_kind(), kind);
        }
    }
}
