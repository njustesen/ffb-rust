use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.Keyword.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    // team level
    MASTER_CHEF, VAMPIRE_LORD,
    // player level
    ANIMAL, BEASTMAN, BIG_GUY, BLITZER, BLOCKER, BUGMAN, CATCHER,
    CONSTRUCT, DWARF, ELF, GHOUL, GNOBLAR, GNOME, GOBLIN, HALFLING,
    HUMAN, LINEMAN, LIZARDMAN, MINOTAUR, OGRE, ORC, RUNNER, SKAVEN,
    SKELETON, SNAKEMAN, SNOTLING, SPAWN, SPECIAL, SQUIRREL, THRALL,
    THROWER, TREEMAN, TROLL, UNDEAD, VAMPIRE, WEREWOLF, WRAITH, YHETEE,
    ZOMBIE,
    // fallback
    ALL, UNKNOWN,
}

impl Keyword {
    pub fn get_name(self) -> &'static str {
        match self {
            Keyword::MASTER_CHEF => "Master Chef",
            Keyword::VAMPIRE_LORD => "Vampire Lord",
            Keyword::ANIMAL => "Animal",
            Keyword::BEASTMAN => "Beastman",
            Keyword::BIG_GUY => "Big Guy",
            Keyword::BLITZER => "Blitzer",
            Keyword::BLOCKER => "Blocker",
            Keyword::BUGMAN => "Bugman",
            Keyword::CATCHER => "Catcher",
            Keyword::CONSTRUCT => "Construct",
            Keyword::DWARF => "Dwarf",
            Keyword::ELF => "Elf",
            Keyword::GHOUL => "Ghoul",
            Keyword::GNOBLAR => "Gnoblar",
            Keyword::GNOME => "Gnome",
            Keyword::GOBLIN => "Goblin",
            Keyword::HALFLING => "Halfling",
            Keyword::HUMAN => "Human",
            Keyword::LINEMAN => "Lineman",
            Keyword::LIZARDMAN => "Lizardman",
            Keyword::MINOTAUR => "Minotaur",
            Keyword::OGRE => "Ogre",
            Keyword::ORC => "Orc",
            Keyword::RUNNER => "Runner",
            Keyword::SKAVEN => "Skaven",
            Keyword::SKELETON => "Skeleton",
            Keyword::SNAKEMAN => "Snakeman",
            Keyword::SNOTLING => "Snotling",
            Keyword::SPAWN => "Spawn",
            Keyword::SPECIAL => "Special",
            Keyword::SQUIRREL => "Squirrel",
            Keyword::THRALL => "Thrall",
            Keyword::THROWER => "Thrower",
            Keyword::TREEMAN => "Treeman",
            Keyword::TROLL => "Troll",
            Keyword::UNDEAD => "Undead",
            Keyword::VAMPIRE => "Vampire",
            Keyword::WEREWOLF => "Werewolf",
            Keyword::WRAITH => "Wraith",
            Keyword::YHETEE => "Yhetee",
            Keyword::ZOMBIE => "Zombie",
            Keyword::ALL => "all",
            Keyword::UNKNOWN => "Unknown",
        }
    }

    pub fn is_can_get_even_with(self) -> bool {
        !matches!(self, Keyword::BIG_GUY | Keyword::BLITZER | Keyword::BLOCKER |
            Keyword::BUGMAN | Keyword::CATCHER | Keyword::LINEMAN | Keyword::RUNNER |
            Keyword::SPECIAL | Keyword::THROWER)
    }

    pub fn for_name(name: &str) -> Self {
        [
            Keyword::MASTER_CHEF, Keyword::VAMPIRE_LORD, Keyword::ANIMAL, Keyword::BEASTMAN,
            Keyword::BIG_GUY, Keyword::BLITZER, Keyword::BLOCKER, Keyword::BUGMAN,
            Keyword::CATCHER, Keyword::CONSTRUCT, Keyword::DWARF, Keyword::ELF,
            Keyword::GHOUL, Keyword::GNOBLAR, Keyword::GNOME, Keyword::GOBLIN,
            Keyword::HALFLING, Keyword::HUMAN, Keyword::LINEMAN, Keyword::LIZARDMAN,
            Keyword::MINOTAUR, Keyword::OGRE, Keyword::ORC, Keyword::RUNNER,
            Keyword::SKAVEN, Keyword::SKELETON, Keyword::SNAKEMAN, Keyword::SNOTLING,
            Keyword::SPAWN, Keyword::SPECIAL, Keyword::SQUIRREL, Keyword::THRALL,
            Keyword::THROWER, Keyword::TREEMAN, Keyword::TROLL, Keyword::UNDEAD,
            Keyword::VAMPIRE, Keyword::WEREWOLF, Keyword::WRAITH, Keyword::YHETEE,
            Keyword::ZOMBIE, Keyword::ALL, Keyword::UNKNOWN,
        ]
        .iter().copied().find(|k| k.get_name().eq_ignore_ascii_case(name))
        .unwrap_or(Keyword::UNKNOWN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_dwarf() {
        assert_eq!(Keyword::for_name("Dwarf"), Keyword::DWARF);
        assert_eq!(Keyword::for_name("dwarf"), Keyword::DWARF);
    }

    #[test]
    fn for_name_unknown_fallback() {
        assert_eq!(Keyword::for_name("nonexistent"), Keyword::UNKNOWN);
    }

    #[test]
    fn big_guy_cannot_get_even_with() {
        assert!(!Keyword::BIG_GUY.is_can_get_even_with());
    }

    #[test]
    fn dwarf_can_get_even_with() {
        assert!(Keyword::DWARF.is_can_get_even_with());
    }

    #[test]
    fn for_name_vampire_returns_vampire() {
        assert_eq!(Keyword::for_name("Vampire"), Keyword::VAMPIRE);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(Keyword::GOBLIN, Keyword::ORC);
        assert_ne!(Keyword::HUMAN, Keyword::ELF);
    }
}
