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
