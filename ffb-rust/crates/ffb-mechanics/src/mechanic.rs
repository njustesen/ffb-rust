/// 1:1 translation of com.fumbbl.ffb.mechanics.Mechanic (interface + inner enum).
pub trait Mechanic {
    fn get_type(&self) -> MechanicType;
    fn get_name(&self) -> &str {
        self.get_type().name()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MechanicType {
    AGILITY,
    GAME,
    INJURY,
    JUMP,
    ON_THE_BALL,
    PASS,
    SKILL,
    STAT,
    TTM,
    SPP,
    APOTHECARY,
    THROW_IN,
    ROLL,
    SETUP,
    STATE,
}

impl MechanicType {
    pub fn name(self) -> &'static str {
        match self {
            MechanicType::AGILITY => "AGILITY",
            MechanicType::GAME => "GAME",
            MechanicType::INJURY => "INJURY",
            MechanicType::JUMP => "JUMP",
            MechanicType::ON_THE_BALL => "ON_THE_BALL",
            MechanicType::PASS => "PASS",
            MechanicType::SKILL => "SKILL",
            MechanicType::STAT => "STAT",
            MechanicType::TTM => "TTM",
            MechanicType::SPP => "SPP",
            MechanicType::APOTHECARY => "APOTHECARY",
            MechanicType::THROW_IN => "THROW_IN",
            MechanicType::ROLL => "ROLL",
            MechanicType::SETUP => "SETUP",
            MechanicType::STATE => "STATE",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "AGILITY" => Some(MechanicType::AGILITY),
            "GAME" => Some(MechanicType::GAME),
            "INJURY" => Some(MechanicType::INJURY),
            "JUMP" => Some(MechanicType::JUMP),
            "ON_THE_BALL" => Some(MechanicType::ON_THE_BALL),
            "PASS" => Some(MechanicType::PASS),
            "SKILL" => Some(MechanicType::SKILL),
            "STAT" => Some(MechanicType::STAT),
            "TTM" => Some(MechanicType::TTM),
            "SPP" => Some(MechanicType::SPP),
            "APOTHECARY" => Some(MechanicType::APOTHECARY),
            "THROW_IN" => Some(MechanicType::THROW_IN),
            "ROLL" => Some(MechanicType::ROLL),
            "SETUP" => Some(MechanicType::SETUP),
            "STATE" => Some(MechanicType::STATE),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mechanic_type_name_returns_uppercase() {
        assert_eq!(MechanicType::AGILITY.name(), "AGILITY");
        assert_eq!(MechanicType::THROW_IN.name(), "THROW_IN");
    }

    #[test]
    fn for_name_round_trips() {
        assert_eq!(MechanicType::for_name("PASS"), Some(MechanicType::PASS));
        assert_eq!(MechanicType::for_name("invalid"), None);
    }

    #[test]
    fn get_name_returns_type_name() {
        struct TestMechanic;
        impl Mechanic for TestMechanic {
            fn get_type(&self) -> MechanicType { MechanicType::AGILITY }
        }
        let m = TestMechanic;
        assert_eq!(m.get_name(), "AGILITY");
    }

    #[test]
    fn all_mechanic_type_names_round_trip_for_name() {
        let variants = [
            MechanicType::AGILITY, MechanicType::GAME, MechanicType::INJURY,
            MechanicType::JUMP, MechanicType::ON_THE_BALL, MechanicType::PASS,
            MechanicType::SKILL, MechanicType::STAT, MechanicType::TTM,
            MechanicType::SPP, MechanicType::APOTHECARY, MechanicType::THROW_IN,
            MechanicType::ROLL, MechanicType::SETUP, MechanicType::STATE,
        ];
        for v in variants {
            let name = v.name();
            assert_eq!(MechanicType::for_name(name), Some(v), "round trip failed for {:?}", v);
        }
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(MechanicType::for_name(""), None);
        assert_eq!(MechanicType::for_name("agility"), None); // case-sensitive
    }

    #[test]
    fn mechanic_type_equality() {
        assert_eq!(MechanicType::PASS, MechanicType::PASS);
        assert_ne!(MechanicType::PASS, MechanicType::SKILL);
    }

    #[test]
    fn mechanic_type_is_copyable() {
        let a = MechanicType::INJURY;
        let b = a;
        assert_eq!(a, b);
    }
}
