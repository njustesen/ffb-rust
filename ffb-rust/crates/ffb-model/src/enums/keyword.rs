/// 1:1 translation of `com.fumbbl.ffb.model.Keyword`.
/// Position keywords define what category a position belongs to (e.g. Lineman, Big Guy, Thrower).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum Keyword {
    MASTER_CHEF,
    VAMPIRE_LORD,
    ANIMAL,
    BEASTMAN,
    BIG_GUY,
    BLITZER,
    BLOCKER,
    BUGMAN,
    CATCHER,
    CONSTRUCT,
    DWARF,
    ELF,
    GHOUL,
    GNOBLAR,
    GNOME,
    GOBLIN,
    HALFLING,
    HUMAN,
    LINEMAN,
    LIZARDMAN,
    MINOTAUR,
    OGRE,
    ORC,
    RUNNER,
    SKAVEN,
    SKELETON,
    SNAKEMAN,
    SNOTLING,
    SPAWN,
    SPECIAL,
    SPITE,
    SQUIRREL,
    THRALL,
    THROWER,
    TREEMAN,
    TROLL,
    UNDEAD,
    VAMPIRE,
    WEREWOLF,
    WRAITH,
    YHETEE,
    ZOAT,
    ZOMBIE,
    ALL,
    UNKNOWN,
}

impl Keyword {
    /// Java: Keyword.getName() — display name of the keyword.
    pub fn get_name(self) -> &'static str {
        match self {
            Keyword::MASTER_CHEF   => "Master Chef",
            Keyword::VAMPIRE_LORD  => "Vampire Lord",
            Keyword::ANIMAL        => "Animal",
            Keyword::BEASTMAN      => "Beastman",
            Keyword::BIG_GUY       => "Big Guy",
            Keyword::BLITZER       => "Blitzer",
            Keyword::BLOCKER       => "Blocker",
            Keyword::BUGMAN        => "Bugman",
            Keyword::CATCHER       => "Catcher",
            Keyword::CONSTRUCT     => "Construct",
            Keyword::DWARF         => "Dwarf",
            Keyword::ELF           => "Elf",
            Keyword::GHOUL         => "Ghoul",
            Keyword::GNOBLAR       => "Gnoblar",
            Keyword::GNOME         => "Gnome",
            Keyword::GOBLIN        => "Goblin",
            Keyword::HALFLING      => "Halfling",
            Keyword::HUMAN         => "Human",
            Keyword::LINEMAN       => "Lineman",
            Keyword::LIZARDMAN     => "Lizardman",
            Keyword::MINOTAUR      => "Minotaur",
            Keyword::OGRE          => "Ogre",
            Keyword::ORC           => "Orc",
            Keyword::RUNNER        => "Runner",
            Keyword::SKAVEN        => "Skaven",
            Keyword::SKELETON      => "Skeleton",
            Keyword::SNAKEMAN      => "Snakeman",
            Keyword::SNOTLING      => "Snotling",
            Keyword::SPAWN         => "Spawn",
            Keyword::SPECIAL       => "Special",
            Keyword::SPITE         => "Spite",
            Keyword::SQUIRREL      => "Squirrel",
            Keyword::THRALL        => "Thrall",
            Keyword::THROWER       => "Thrower",
            Keyword::TREEMAN       => "Treeman",
            Keyword::TROLL         => "Troll",
            Keyword::UNDEAD        => "Undead",
            Keyword::VAMPIRE       => "Vampire",
            Keyword::WEREWOLF      => "Werewolf",
            Keyword::WRAITH        => "Wraith",
            Keyword::YHETEE        => "Yhetee",
            Keyword::ZOAT          => "Zoat",
            Keyword::ZOMBIE        => "Zombie",
            Keyword::ALL           => "all",
            Keyword::UNKNOWN       => "Unknown",
        }
    }

    /// Java: Keyword.forName(String) — case-insensitive lookup by display name.
    pub fn for_name(name: &str) -> Keyword {
        let lower = name.to_lowercase();
        match lower.as_str() {
            "master chef"  => Keyword::MASTER_CHEF,
            "vampire lord" => Keyword::VAMPIRE_LORD,
            "animal"       => Keyword::ANIMAL,
            "beastman"     => Keyword::BEASTMAN,
            "big guy"      => Keyword::BIG_GUY,
            "blitzer"      => Keyword::BLITZER,
            "blocker"      => Keyword::BLOCKER,
            "bugman"       => Keyword::BUGMAN,
            "catcher"      => Keyword::CATCHER,
            "construct"    => Keyword::CONSTRUCT,
            "dwarf"        => Keyword::DWARF,
            "elf"          => Keyword::ELF,
            "ghoul"        => Keyword::GHOUL,
            "gnoblar"      => Keyword::GNOBLAR,
            "gnome"        => Keyword::GNOME,
            "goblin"       => Keyword::GOBLIN,
            "halfling"     => Keyword::HALFLING,
            "human"        => Keyword::HUMAN,
            "lineman"      => Keyword::LINEMAN,
            "lizardman"    => Keyword::LIZARDMAN,
            "minotaur"     => Keyword::MINOTAUR,
            "ogre"         => Keyword::OGRE,
            "orc"          => Keyword::ORC,
            "runner"       => Keyword::RUNNER,
            "skaven"       => Keyword::SKAVEN,
            "skeleton"     => Keyword::SKELETON,
            "snakeman"     => Keyword::SNAKEMAN,
            "snotling"     => Keyword::SNOTLING,
            "spawn"        => Keyword::SPAWN,
            "special"      => Keyword::SPECIAL,
            "spite"        => Keyword::SPITE,
            "squirrel"     => Keyword::SQUIRREL,
            "thrall"       => Keyword::THRALL,
            "thrower"      => Keyword::THROWER,
            "treeman"      => Keyword::TREEMAN,
            "troll"        => Keyword::TROLL,
            "undead"       => Keyword::UNDEAD,
            "vampire"      => Keyword::VAMPIRE,
            "werewolf"     => Keyword::WEREWOLF,
            "wraith"       => Keyword::WRAITH,
            "yhetee"       => Keyword::YHETEE,
            "zoat"         => Keyword::ZOAT,
            "zombie"       => Keyword::ZOMBIE,
            "all"          => Keyword::ALL,
            _              => Keyword::UNKNOWN,
        }
    }

    /// Java: Keyword.isCanGetEvenWith() — whether Getting Even can target this position type.
    /// Most keywords return true; BIG_GUY, BLOCKER, BLITZER, BUGMAN, CATCHER, RUNNER, SPECIAL,
    /// LINEMAN, THROWER return false (denoted by `false` in the Java constructor).
    pub fn is_can_get_even_with(self) -> bool {
        !matches!(
            self,
            Keyword::BIG_GUY
                | Keyword::BLITZER
                | Keyword::BLOCKER
                | Keyword::BUGMAN
                | Keyword::CATCHER
                | Keyword::LINEMAN
                | Keyword::RUNNER
                | Keyword::SPECIAL
                | Keyword::THROWER
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_lineman_round_trip() {
        let kw = Keyword::for_name("Lineman");
        assert_eq!(kw, Keyword::LINEMAN);
        assert_eq!(kw.get_name(), "Lineman");
    }

    #[test]
    fn for_name_case_insensitive() {
        assert_eq!(Keyword::for_name("big guy"), Keyword::BIG_GUY);
        assert_eq!(Keyword::for_name("BIG GUY"), Keyword::BIG_GUY);
        assert_eq!(Keyword::for_name("Vampire Lord"), Keyword::VAMPIRE_LORD);
    }

    #[test]
    fn for_name_unknown_returns_unknown() {
        assert_eq!(Keyword::for_name("not a keyword"), Keyword::UNKNOWN);
    }

    #[test]
    fn all_named_keywords_have_unique_names() {
        let all = [
            Keyword::MASTER_CHEF, Keyword::VAMPIRE_LORD, Keyword::ANIMAL, Keyword::BEASTMAN,
            Keyword::BIG_GUY, Keyword::BLITZER, Keyword::BLOCKER, Keyword::BUGMAN,
            Keyword::CATCHER, Keyword::CONSTRUCT, Keyword::DWARF, Keyword::ELF,
            Keyword::GHOUL, Keyword::GNOBLAR, Keyword::GNOME, Keyword::GOBLIN,
            Keyword::HALFLING, Keyword::HUMAN, Keyword::LINEMAN, Keyword::LIZARDMAN,
            Keyword::MINOTAUR, Keyword::OGRE, Keyword::ORC, Keyword::RUNNER,
            Keyword::SKAVEN, Keyword::SKELETON, Keyword::SNAKEMAN, Keyword::SNOTLING,
            Keyword::SPAWN, Keyword::SPECIAL, Keyword::SPITE, Keyword::SQUIRREL,
            Keyword::THRALL, Keyword::THROWER, Keyword::TREEMAN, Keyword::TROLL,
            Keyword::UNDEAD, Keyword::VAMPIRE, Keyword::WEREWOLF, Keyword::WRAITH,
            Keyword::YHETEE, Keyword::ZOAT, Keyword::ZOMBIE, Keyword::ALL,
        ];
        let names: std::collections::HashSet<_> = all.iter().map(|k| k.get_name()).collect();
        assert_eq!(names.len(), all.len(), "all keywords must have unique names");
    }

    #[test]
    fn is_can_get_even_with_big_guy_false() {
        assert!(!Keyword::BIG_GUY.is_can_get_even_with());
        assert!(!Keyword::LINEMAN.is_can_get_even_with());
        assert!(!Keyword::BLITZER.is_can_get_even_with());
    }

    #[test]
    fn is_can_get_even_with_vampire_true() {
        assert!(Keyword::VAMPIRE.is_can_get_even_with());
        assert!(Keyword::ZOMBIE.is_can_get_even_with());
        assert!(Keyword::TROLL.is_can_get_even_with());
    }

    #[test]
    fn for_name_all_variants_roundtrip() {
        let cases = [
            ("Lineman", Keyword::LINEMAN),
            ("Vampire Lord", Keyword::VAMPIRE_LORD),
            ("Big Guy", Keyword::BIG_GUY),
            ("Blocker", Keyword::BLOCKER),
            ("Thrall", Keyword::THRALL),
        ];
        for (name, expected) in cases {
            assert_eq!(Keyword::for_name(name), expected, "failed for {}", name);
        }
    }
}
