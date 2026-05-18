/// Race roster system (T-59).
///
/// Provides hardcoded position stats for the 6 most common Blood Bowl 2025
/// races and a `make_team` helper that builds a ready-to-play `Team`.
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::skills::{SkillId, SkillSet};
use ffb_core::types::{PlayerId, SpecialRule, TeamId};

// ── PositionDef ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PositionDef {
    pub position_id: &'static str,
    pub name: &'static str,
    pub ma: u8,
    pub st: u8,
    pub ag: u8,
    pub av: u8,
    pub pa: Option<u8>,
    pub skills: &'static [SkillId],
    pub max_count: u8,
}

// ── Roster tables ─────────────────────────────────────────────────────────────

// Human
const HUMAN_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "human_lineman",
        name: "Lineman",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(5),
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "human_thrower",
        name: "Thrower",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(3),
        skills: &[SkillId::Pass, SkillId::SureHands],
        max_count: 2,
    },
    PositionDef {
        position_id: "human_catcher",
        name: "Catcher",
        ma: 8, st: 2, ag: 4, av: 8, pa: Some(5),
        skills: &[SkillId::Catch, SkillId::Dodge],
        max_count: 4,
    },
    PositionDef {
        position_id: "human_blitzer",
        name: "Blitzer",
        ma: 7, st: 3, ag: 4, av: 9, pa: Some(5),
        skills: &[SkillId::Block],
        max_count: 4,
    },
];

// Orc
const ORC_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "orc_black_orc",
        name: "Black Orc",
        ma: 4, st: 4, ag: 4, av: 10, pa: Some(6),
        skills: &[],
        max_count: 4,
    },
    PositionDef {
        position_id: "orc_thrower",
        name: "Orc Thrower",
        ma: 5, st: 3, ag: 4, av: 9, pa: Some(4),
        skills: &[SkillId::Pass, SkillId::SureHands],
        max_count: 2,
    },
    PositionDef {
        position_id: "orc_blitzer",
        name: "Orc Blitzer",
        ma: 6, st: 3, ag: 3, av: 9, pa: Some(5),
        skills: &[SkillId::Block],
        max_count: 4,
    },
    PositionDef {
        position_id: "orc_big_un",
        name: "Big Un",
        ma: 5, st: 4, ag: 4, av: 10, pa: Some(6),
        skills: &[],
        max_count: 2,
    },
    PositionDef {
        position_id: "orc_troll",
        name: "Troll",
        ma: 4, st: 5, ag: 5, av: 10, pa: Some(6),
        skills: &[SkillId::Loner, SkillId::ReallyStupid, SkillId::Regeneration, SkillId::ThrowTeamMate],
        max_count: 1,
    },
];

// Dark Elf
const DARK_ELF_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "dark_elf_lineman",
        name: "Dark Elf Lineman",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(5),
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "dark_elf_runner",
        name: "Runner",
        ma: 7, st: 3, ag: 4, av: 8, pa: Some(4),
        skills: &[SkillId::DumpOff],
        max_count: 2,
    },
    PositionDef {
        position_id: "dark_elf_witch_elf",
        name: "Witch Elf",
        ma: 7, st: 3, ag: 4, av: 8, pa: Some(5),
        skills: &[SkillId::Dodge, SkillId::Frenzy, SkillId::JumpUp],
        max_count: 4,
    },
    PositionDef {
        position_id: "dark_elf_blitzer",
        name: "Dark Elf Blitzer",
        ma: 7, st: 3, ag: 4, av: 9, pa: Some(5),
        skills: &[SkillId::Block, SkillId::Dodge],
        max_count: 4,
    },
];

// Skaven
const SKAVEN_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "skaven_lineman",
        name: "Skaven Lineman",
        ma: 7, st: 3, ag: 4, av: 8, pa: Some(5),
        skills: &[],
        max_count: 6,
    },
    PositionDef {
        position_id: "skaven_thrower",
        name: "Skaven Thrower",
        ma: 7, st: 3, ag: 4, av: 8, pa: Some(4),
        skills: &[SkillId::Pass, SkillId::SureHands],
        max_count: 2,
    },
    PositionDef {
        position_id: "skaven_gutter_runner",
        name: "Gutter Runner",
        ma: 9, st: 2, ag: 5, av: 7, pa: Some(5),
        skills: &[SkillId::Dodge],
        max_count: 4,
    },
    PositionDef {
        position_id: "skaven_stormvermin",
        name: "Stormvermin",
        ma: 7, st: 4, ag: 4, av: 9, pa: Some(5),
        skills: &[SkillId::Block],
        max_count: 2,
    },
    PositionDef {
        position_id: "skaven_rat_ogre",
        name: "Rat Ogre",
        ma: 6, st: 5, ag: 4, av: 9, pa: Some(6),
        skills: &[
            SkillId::Loner,
            SkillId::Frenzy,
            SkillId::MightyBlow,
            SkillId::PrehensileTail,
        ],
        max_count: 1,
    },
];

// Undead
const UNDEAD_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "undead_skeleton",
        name: "Skeleton",
        ma: 5, st: 3, ag: 5, av: 8, pa: Some(6),
        skills: &[],
        max_count: 6,
    },
    PositionDef {
        position_id: "undead_zombie",
        name: "Zombie",
        ma: 4, st: 3, ag: 5, av: 9, pa: Some(6),
        skills: &[],
        max_count: 4,
    },
    PositionDef {
        position_id: "undead_ghoul",
        name: "Ghoul",
        ma: 7, st: 3, ag: 4, av: 8, pa: Some(4),
        skills: &[SkillId::Dodge],
        max_count: 2,
    },
    PositionDef {
        position_id: "undead_wight",
        name: "Wight",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(5),
        skills: &[SkillId::Block, SkillId::Regeneration],
        max_count: 2,
    },
    PositionDef {
        position_id: "undead_mummy",
        name: "Mummy",
        ma: 3, st: 5, ag: 5, av: 10, pa: Some(6),
        skills: &[SkillId::MightyBlow, SkillId::Regeneration],
        max_count: 2,
    },
];

// Dwarf
const DWARF_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "dwarf_blocker",
        name: "Dwarf Blocker",
        ma: 4, st: 3, ag: 4, av: 10, pa: Some(5),
        skills: &[SkillId::Block, SkillId::Tackle, SkillId::ThickSkull],
        max_count: 6,
    },
    PositionDef {
        position_id: "dwarf_runner",
        name: "Dwarf Runner",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(4),
        skills: &[SkillId::SureHands, SkillId::ThickSkull],
        max_count: 2,
    },
    PositionDef {
        position_id: "dwarf_blitzer",
        name: "Dwarf Blitzer",
        ma: 5, st: 3, ag: 4, av: 10, pa: Some(5),
        skills: &[SkillId::Block, SkillId::Dauntless, SkillId::ThickSkull],
        max_count: 4,
    },
    PositionDef {
        position_id: "dwarf_troll_slayer",
        name: "Troll Slayer",
        ma: 5, st: 3, ag: 4, av: 9, pa: Some(6),
        skills: &[SkillId::Block, SkillId::Dauntless, SkillId::Frenzy, SkillId::ThickSkull],
        max_count: 4,
    },
    PositionDef {
        position_id: "dwarf_deathroller",
        name: "Deathroller",
        ma: 4, st: 7, ag: 5, av: 11, pa: Some(6),
        skills: &[
            SkillId::Loner,
            SkillId::BreakTackle,
            SkillId::Juggernaut,
            SkillId::MightyBlow,
            SkillId::ThickSkull,
        ],
        max_count: 1,
    },
];

// Amazon
const AMAZON_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "amazon_linewoman",
        name: "Linewoman",
        ma: 6, st: 3, ag: 2, av: 7, pa: None,
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "amazon_blitzer",
        name: "Amazon Blitzer",
        ma: 6, st: 3, ag: 2, av: 7, pa: None,
        skills: &[SkillId::Block, SkillId::Dodge],
        max_count: 4,
    },
    PositionDef {
        position_id: "amazon_thrower",
        name: "Amazon Thrower",
        ma: 6, st: 3, ag: 2, av: 7, pa: Some(4),
        skills: &[SkillId::Pass],
        max_count: 2,
    },
    PositionDef {
        position_id: "amazon_catcher",
        name: "Amazon Catcher",
        ma: 8, st: 2, ag: 2, av: 7, pa: None,
        skills: &[SkillId::Catch, SkillId::Dodge],
        max_count: 4,
    },
];

// Chaos
const CHAOS_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "chaos_beastman",
        name: "Beastman",
        ma: 6, st: 3, ag: 3, av: 8, pa: None,
        skills: &[SkillId::Horns],
        max_count: 8,
    },
    PositionDef {
        position_id: "chaos_warrior",
        name: "Chaos Warrior",
        ma: 5, st: 4, ag: 3, av: 9, pa: None,
        skills: &[],
        max_count: 4,
    },
    PositionDef {
        position_id: "chaos_minotaur",
        name: "Minotaur",
        ma: 5, st: 5, ag: 4, av: 9, pa: None,
        skills: &[
            SkillId::Frenzy,
            SkillId::MightyBlow,
            SkillId::ThickSkull,
            SkillId::WildAnimal,
            SkillId::Horns,
        ],
        max_count: 1,
    },
];

// Norse
const NORSE_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "norse_lineman",
        name: "Norse Lineman",
        ma: 6, st: 3, ag: 3, av: 7, pa: None,
        skills: &[SkillId::Block],
        max_count: 6,
    },
    PositionDef {
        position_id: "norse_runner",
        name: "Norse Runner",
        ma: 7, st: 3, ag: 3, av: 7, pa: None,
        skills: &[SkillId::Dodge],
        max_count: 2,
    },
    PositionDef {
        position_id: "norse_berserker",
        name: "Berserker",
        ma: 6, st: 3, ag: 3, av: 8, pa: None,
        skills: &[SkillId::Block, SkillId::Frenzy],
        max_count: 2,
    },
    PositionDef {
        position_id: "norse_snow_troll",
        name: "Snow Troll",
        ma: 5, st: 5, ag: 5, av: 9, pa: None,
        skills: &[
            SkillId::Loner,
            SkillId::Frenzy,
            SkillId::Claws,
            SkillId::ThickSkull,
            SkillId::WildAnimal,
            SkillId::DisturbingPresence,
        ],
        max_count: 1,
    },
];

// Halfling
const HALFLING_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "halfling_halfling",
        name: "Halfling",
        ma: 5, st: 2, ag: 3, av: 6, pa: None,
        skills: &[SkillId::Dodge, SkillId::RightStuff, SkillId::Stunty],
        max_count: 9,
    },
    PositionDef {
        position_id: "halfling_treeman",
        name: "Treeman",
        ma: 2, st: 6, ag: 5, av: 11, pa: None,
        skills: &[
            SkillId::MightyBlow,
            SkillId::StandFirm,
            SkillId::StrongArm,
            SkillId::TakeRoot,
            SkillId::ThickSkull,
            SkillId::ThrowTeamMate,
        ],
        max_count: 2,
    },
];

// High Elf
const HIGH_ELF_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "high_elf_lineman",
        name: "High Elf Lineman",
        ma: 6, st: 3, ag: 2, av: 8, pa: None,
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "high_elf_thrower",
        name: "High Elf Thrower",
        ma: 6, st: 3, ag: 2, av: 8, pa: Some(3),
        skills: &[SkillId::Pass, SkillId::SafePairOfHands],
        max_count: 2,
    },
    PositionDef {
        position_id: "high_elf_catcher",
        name: "High Elf Catcher",
        ma: 8, st: 3, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Catch, SkillId::NervesOfSteel],
        max_count: 4,
    },
    PositionDef {
        position_id: "high_elf_blitzer",
        name: "High Elf Blitzer",
        ma: 7, st: 3, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Block, SkillId::SideStep],
        max_count: 4,
    },
];

// Goblin
const GOBLIN_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "goblin_goblin",
        name: "Goblin",
        ma: 6, st: 2, ag: 3, av: 7, pa: None,
        skills: &[SkillId::Dodge, SkillId::RightStuff, SkillId::Stunty],
        max_count: 9,
    },
    PositionDef {
        position_id: "goblin_troll",
        name: "Goblin Troll",
        ma: 4, st: 5, ag: 5, av: 9, pa: Some(6),
        skills: &[SkillId::Loner, SkillId::AlwaysHungry, SkillId::MightyBlow,
                  SkillId::ReallyStupid, SkillId::Regeneration, SkillId::ThrowTeamMate],
        max_count: 2,
    },
];

// Lizardman
const LIZARDMAN_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "lizardman_skink",
        name: "Skink",
        ma: 8, st: 2, ag: 3, av: 8, pa: None,
        skills: &[SkillId::Dodge, SkillId::PrehensileTail, SkillId::Stunty],
        max_count: 6,
    },
    PositionDef {
        position_id: "lizardman_saurus",
        name: "Saurus",
        ma: 6, st: 4, ag: 5, av: 10, pa: None,
        skills: &[],
        max_count: 6,
    },
    PositionDef {
        position_id: "lizardman_kroxigor",
        name: "Kroxigor",
        ma: 6, st: 5, ag: 5, av: 10, pa: None,
        skills: &[SkillId::Loner, SkillId::MightyBlow, SkillId::PrehensileTail, SkillId::ThickSkull],
        max_count: 1,
    },
];

// Common (Pro) Elf
const ELF_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "elf_lineman",
        name: "Elf Lineman",
        ma: 6, st: 3, ag: 2, av: 8, pa: Some(5),
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "elf_thrower",
        name: "Elf Thrower",
        ma: 6, st: 3, ag: 2, av: 8, pa: Some(3),
        skills: &[SkillId::Pass],
        max_count: 2,
    },
    PositionDef {
        position_id: "elf_catcher",
        name: "Elf Catcher",
        ma: 8, st: 3, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Catch, SkillId::Dodge],
        max_count: 4,
    },
    PositionDef {
        position_id: "elf_blitzer",
        name: "Elf Blitzer",
        ma: 7, st: 3, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Block, SkillId::Dodge],
        max_count: 4,
    },
];

// Wood Elf
const WOOD_ELF_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "wood_elf_lineman",
        name: "Wood Elf Lineman",
        ma: 7, st: 3, ag: 2, av: 8, pa: Some(5),
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "wood_elf_thrower",
        name: "Wood Elf Thrower",
        ma: 7, st: 3, ag: 2, av: 8, pa: Some(3),
        skills: &[SkillId::Pass],
        max_count: 2,
    },
    PositionDef {
        position_id: "wood_elf_catcher",
        name: "Wood Elf Catcher",
        ma: 8, st: 2, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Catch, SkillId::Dodge],
        max_count: 4,
    },
    PositionDef {
        position_id: "wood_elf_wardancer",
        name: "Wardancer",
        ma: 8, st: 3, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Block, SkillId::Dodge, SkillId::Leap],
        max_count: 2,
    },
    PositionDef {
        position_id: "wood_elf_treeman",
        name: "Treeman",
        ma: 2, st: 6, ag: 5, av: 11, pa: None,
        skills: &[SkillId::MightyBlow, SkillId::StandFirm, SkillId::StrongArm,
                  SkillId::TakeRoot, SkillId::ThickSkull, SkillId::ThrowTeamMate],
        max_count: 1,
    },
];

// Chaos Dwarf
const CHAOS_DWARF_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "chaos_dwarf_chaos_dwarf_blocker",
        name: "Chaos Dwarf Blocker",
        ma: 4, st: 3, ag: 4, av: 10, pa: Some(6),
        skills: &[SkillId::Block, SkillId::Tackle, SkillId::ThickSkull],
        max_count: 6,
    },
    PositionDef {
        position_id: "chaos_dwarf_hobgoblin",
        name: "Hobgoblin",
        ma: 6, st: 3, ag: 4, av: 8, pa: None,
        skills: &[],
        max_count: 4,
    },
    PositionDef {
        position_id: "chaos_dwarf_bull_centaur",
        name: "Bull Centaur",
        ma: 6, st: 4, ag: 4, av: 10, pa: Some(6),
        skills: &[SkillId::ThickSkull, SkillId::Sprint, SkillId::SureFeet],
        max_count: 2,
    },
    PositionDef {
        position_id: "chaos_dwarf_minotaur",
        name: "Minotaur",
        ma: 5, st: 5, ag: 4, av: 9, pa: None,
        skills: &[SkillId::Frenzy, SkillId::Loner, SkillId::MightyBlow,
                  SkillId::ThickSkull, SkillId::WildAnimal, SkillId::Horns],
        max_count: 1,
    },
];

// Chaos Pact
const CHAOS_PACT_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "chaos_pact_marauder",
        name: "Marauder",
        ma: 6, st: 3, ag: 3, av: 8, pa: None,
        skills: &[],
        max_count: 7,
    },
    PositionDef {
        position_id: "chaos_pact_goblin",
        name: "Goblin Renegade",
        ma: 6, st: 2, ag: 3, av: 7, pa: None,
        skills: &[SkillId::Dodge, SkillId::RightStuff, SkillId::Stunty],
        max_count: 1,
    },
    PositionDef {
        position_id: "chaos_pact_skaven",
        name: "Skaven Renegade",
        ma: 7, st: 3, ag: 3, av: 8, pa: None,
        skills: &[],
        max_count: 1,
    },
    PositionDef {
        position_id: "chaos_pact_dark_elf",
        name: "Dark Elf Renegade",
        ma: 6, st: 3, ag: 3, av: 9, pa: None,
        skills: &[],
        max_count: 1,
    },
    PositionDef {
        position_id: "chaos_pact_ogre",
        name: "Ogre",
        ma: 5, st: 5, ag: 4, av: 10, pa: Some(6),
        skills: &[SkillId::Loner, SkillId::BoneHead, SkillId::MightyBlow,
                  SkillId::ThickSkull, SkillId::ThrowTeamMate],
        max_count: 1,
    },
];

// Necromantic
const NECROMANTIC_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "necromantic_zombie",
        name: "Zombie",
        ma: 4, st: 3, ag: 5, av: 9, pa: Some(6),
        skills: &[],
        max_count: 6,
    },
    PositionDef {
        position_id: "necromantic_ghoul",
        name: "Ghoul",
        ma: 7, st: 3, ag: 4, av: 8, pa: Some(4),
        skills: &[SkillId::Dodge],
        max_count: 2,
    },
    PositionDef {
        position_id: "necromantic_werewolf",
        name: "Werewolf",
        ma: 8, st: 3, ag: 3, av: 8, pa: Some(5),
        skills: &[SkillId::Claws, SkillId::Frenzy, SkillId::Regeneration],
        max_count: 2,
    },
    PositionDef {
        position_id: "necromantic_flesh_golem",
        name: "Flesh Golem",
        ma: 4, st: 4, ag: 4, av: 10, pa: Some(6),
        skills: &[SkillId::Regeneration, SkillId::StandFirm, SkillId::ThickSkull],
        max_count: 2,
    },
    PositionDef {
        position_id: "necromantic_wight",
        name: "Wight",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(5),
        skills: &[SkillId::Block, SkillId::Regeneration],
        max_count: 2,
    },
];

// Nurgle
const NURGLE_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "nurgle_rotter",
        name: "Rotter",
        ma: 5, st: 3, ag: 4, av: 9, pa: None,
        skills: &[SkillId::Decay, SkillId::NurglesRot],
        max_count: 6,
    },
    PositionDef {
        position_id: "nurgle_pestigor",
        name: "Pestigor",
        ma: 6, st: 3, ag: 3, av: 9, pa: None,
        skills: &[SkillId::Horns, SkillId::NurglesRot, SkillId::Regeneration],
        max_count: 4,
    },
    PositionDef {
        position_id: "nurgle_warrior",
        name: "Nurgle Warrior",
        ma: 4, st: 4, ag: 4, av: 10, pa: None,
        skills: &[SkillId::DisturbingPresence, SkillId::FoulAppearance, SkillId::Regeneration],
        max_count: 4,
    },
    PositionDef {
        position_id: "nurgle_beast",
        name: "Beast of Nurgle",
        ma: 4, st: 5, ag: 5, av: 10, pa: None,
        skills: &[SkillId::Loner, SkillId::DisturbingPresence, SkillId::FoulAppearance,
                  SkillId::MightyBlow, SkillId::Regeneration, SkillId::Tentacles],
        max_count: 1,
    },
];

// Ogre
const OGRE_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "ogre_snotling",
        name: "Snotling",
        ma: 5, st: 1, ag: 3, av: 5, pa: None,
        skills: &[SkillId::Dodge, SkillId::RightStuff, SkillId::Stunty, SkillId::Titchy],
        max_count: 6,
    },
    PositionDef {
        position_id: "ogre_ogre",
        name: "Ogre",
        ma: 5, st: 5, ag: 4, av: 10, pa: Some(6),
        skills: &[SkillId::Loner, SkillId::BoneHead, SkillId::MightyBlow,
                  SkillId::ThickSkull, SkillId::ThrowTeamMate],
        max_count: 6,
    },
];

// Slann
const SLANN_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "slann_lineman",
        name: "Slann Lineman",
        ma: 6, st: 3, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Leap],
        max_count: 8,
    },
    PositionDef {
        position_id: "slann_catcher",
        name: "Slann Catcher",
        ma: 8, st: 2, ag: 2, av: 8, pa: None,
        skills: &[SkillId::Catch, SkillId::Dodge, SkillId::Leap, SkillId::VeryLongLegs],
        max_count: 4,
    },
    PositionDef {
        position_id: "slann_blitzer",
        name: "Slann Blitzer",
        ma: 7, st: 3, ag: 2, av: 9, pa: None,
        skills: &[SkillId::Block, SkillId::Dodge, SkillId::Leap],
        max_count: 4,
    },
    PositionDef {
        position_id: "slann_kroxigor",
        name: "Kroxigor",
        ma: 6, st: 5, ag: 5, av: 10, pa: None,
        skills: &[SkillId::Loner, SkillId::MightyBlow, SkillId::PrehensileTail, SkillId::ThickSkull],
        max_count: 1,
    },
];

// Vampire
const VAMPIRE_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "vampire_thrall",
        name: "Human Thrall",
        ma: 6, st: 3, ag: 4, av: 9, pa: None,
        skills: &[],
        max_count: 6,
    },
    PositionDef {
        position_id: "vampire_vampire",
        name: "Vampire",
        ma: 6, st: 4, ag: 2, av: 9, pa: Some(4),
        skills: &[SkillId::Bloodlust, SkillId::HypnoticGaze, SkillId::Regeneration],
        max_count: 6,
    },
];

// Underworld Denizens
const UNDERWORLD_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "underworld_goblin",
        name: "Underworld Goblin",
        ma: 6, st: 2, ag: 3, av: 7, pa: None,
        skills: &[SkillId::Dodge, SkillId::RightStuff, SkillId::Stunty],
        max_count: 7,
    },
    PositionDef {
        position_id: "underworld_skaven",
        name: "Underworld Skaven",
        ma: 7, st: 3, ag: 4, av: 8, pa: None,
        skills: &[],
        max_count: 2,
    },
    PositionDef {
        position_id: "underworld_blitzer",
        name: "Underworld Blitzer",
        ma: 7, st: 3, ag: 4, av: 9, pa: None,
        skills: &[SkillId::Block, SkillId::Horns],
        max_count: 2,
    },
    PositionDef {
        position_id: "underworld_troll",
        name: "Underworld Troll",
        ma: 4, st: 5, ag: 5, av: 9, pa: Some(6),
        skills: &[SkillId::Loner, SkillId::AlwaysHungry, SkillId::MightyBlow,
                  SkillId::ReallyStupid, SkillId::Regeneration, SkillId::ThrowTeamMate],
        max_count: 1,
    },
];

// Khemri
const KHEMRI_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "khemri_skeleton",
        name: "Skeleton",
        ma: 5, st: 3, ag: 5, av: 8, pa: Some(6),
        skills: &[],
        max_count: 6,
    },
    PositionDef {
        position_id: "khemri_blitz_ra",
        name: "Blitz-Ra",
        ma: 6, st: 3, ag: 4, av: 9, pa: None,
        skills: &[SkillId::Block, SkillId::Regeneration],
        max_count: 2,
    },
    PositionDef {
        position_id: "khemri_thro_ra",
        name: "Thro-Ra",
        ma: 6, st: 3, ag: 4, av: 8, pa: Some(4),
        skills: &[SkillId::Pass, SkillId::SureHands, SkillId::Regeneration],
        max_count: 2,
    },
    PositionDef {
        position_id: "khemri_tomb_guardian",
        name: "Tomb Guardian",
        ma: 4, st: 5, ag: 5, av: 10, pa: Some(6),
        skills: &[SkillId::MightyBlow, SkillId::Regeneration, SkillId::ThickSkull],
        max_count: 4,
    },
];

// Nippon (Equivalent to Human for game purposes, different fluff)
const NIPPON_ROSTER: &[PositionDef] = &[
    PositionDef {
        position_id: "nippon_lineman",
        name: "Nippon Lineman",
        ma: 6, st: 3, ag: 4, av: 9, pa: None,
        skills: &[],
        max_count: 8,
    },
    PositionDef {
        position_id: "nippon_thrower",
        name: "Nippon Thrower",
        ma: 6, st: 3, ag: 4, av: 9, pa: Some(3),
        skills: &[SkillId::Pass, SkillId::SureHands],
        max_count: 2,
    },
    PositionDef {
        position_id: "nippon_catcher",
        name: "Nippon Catcher",
        ma: 8, st: 2, ag: 4, av: 8, pa: None,
        skills: &[SkillId::Catch, SkillId::Dodge],
        max_count: 4,
    },
    PositionDef {
        position_id: "nippon_blitzer",
        name: "Nippon Blitzer",
        ma: 7, st: 3, ag: 4, av: 9, pa: None,
        skills: &[SkillId::Block],
        max_count: 4,
    },
];

// ── RosterLibrary ─────────────────────────────────────────────────────────────

pub struct RosterLibrary;

impl RosterLibrary {
    /// Return the position definitions for the given race (case-insensitive).
    pub fn positions(race: &str) -> Option<&'static [PositionDef]> {
        match race.to_lowercase().as_str() {
            "human" => Some(HUMAN_ROSTER),
            "orc" => Some(ORC_ROSTER),
            "dark elf" | "dark_elf" | "darkelf" => Some(DARK_ELF_ROSTER),
            "skaven" => Some(SKAVEN_ROSTER),
            "undead" => Some(UNDEAD_ROSTER),
            "dwarf" => Some(DWARF_ROSTER),
            "amazon" => Some(AMAZON_ROSTER),
            "chaos" => Some(CHAOS_ROSTER),
            "norse" => Some(NORSE_ROSTER),
            "halfling" => Some(HALFLING_ROSTER),
            "high elf" | "high_elf" | "highelf" => Some(HIGH_ELF_ROSTER),
            "goblin" => Some(GOBLIN_ROSTER),
            "lizardman" | "lizardmen" => Some(LIZARDMAN_ROSTER),
            "elf" | "pro elf" | "common elf" => Some(ELF_ROSTER),
            "wood elf" | "wood_elf" | "woodelf" => Some(WOOD_ELF_ROSTER),
            "chaos dwarf" | "chaos_dwarf" | "chaosdwarf" => Some(CHAOS_DWARF_ROSTER),
            "chaos pact" | "chaos_pact" | "chaospact" => Some(CHAOS_PACT_ROSTER),
            "necromantic" => Some(NECROMANTIC_ROSTER),
            "nurgle" => Some(NURGLE_ROSTER),
            "ogre" => Some(OGRE_ROSTER),
            "slann" => Some(SLANN_ROSTER),
            "vampire" => Some(VAMPIRE_ROSTER),
            "underworld" | "underworld denizens" => Some(UNDERWORLD_ROSTER),
            "khemri" => Some(KHEMRI_ROSTER),
            "nippon" => Some(NIPPON_ROSTER),
            _ => None,
        }
    }
}

// ── make_team ─────────────────────────────────────────────────────────────────

/// Build a standard 11-player team from the given race roster.
///
/// Players are allocated from position slots in definition order, respecting
/// `max_count`, until 11 players (or the full roster) are placed.
/// Jersey numbers run from 1 upwards.
pub fn make_team(
    race: &str,
    team_id: TeamId,
    id: &str,
    name: &str,
    rerolls: u8,
) -> Result<Team, String> {
    let positions = RosterLibrary::positions(race)
        .ok_or_else(|| format!("Unknown race: {race}"))?;

    let apothecary = !matches!(race.to_lowercase().as_str(), "undead");
    let mut team = Team::new(id.to_string(), name.to_string(), race.to_string(), rerolls, apothecary);

    let mut jersey = 1u8;
    let target = 11usize;
    let mut placed = 0usize;

    // First pass: fill positions in order, up to their max_count
    // We aim to hit exactly 11 players by filling cheaper/lineman slots last.
    // We do two passes:
    //   Pass 1 — specialty positions (non-max positions first)
    //   Pass 2 — fill remainder with the first position (lineman)

    // Collect (position, count_to_place)
    let mut alloc: Vec<(&PositionDef, u8)> = positions
        .iter()
        .map(|p| (p, 0u8))
        .collect();

    // Fill specialties (positions after the first) up to their max
    for (pos, count) in alloc.iter_mut().skip(1) {
        if placed >= target {
            break;
        }
        let add = (pos.max_count as usize).min(target - placed) as u8;
        *count = add;
        placed += add as usize;
    }

    // Fill remaining slots with the lineman (first position)
    if placed < target {
        let remaining = (target - placed) as u8;
        alloc[0].1 = remaining.min(alloc[0].0.max_count);
        placed += alloc[0].1 as usize;
    }

    // Build players from allocation
    for (pos, count) in &alloc {
        let skills: SkillSet = pos.skills.iter().copied().collect();
        for _ in 0..*count {
            let player_id = PlayerId(format!("{}_{}_{jersey}", id, pos.position_id));
            let stats = PlayerStats::new(pos.ma, pos.st, pos.ag, pos.av, pos.pa);
            let player = Player::new(
                player_id,
                format!("{} #{jersey}", pos.name),
                pos.position_id.to_string(),
                team_id,
                jersey,
                stats,
                skills.clone(),
            );
            team.add_player(player);
            jersey += 1;
        }
    }

    if placed == 0 {
        return Err(format!("Race {race} produced 0 players"));
    }

    // Assign team special rules based on race
    match race.to_lowercase().as_str() {
        "necromantic" | "undead" | "khemri" => {
            team.special_rules.push(SpecialRule::MastersOfUndeath);
        }
        "goblin" | "halfling" => {
            team.special_rules.push(SpecialRule::Swarming);
        }
        "chaos" | "chaos pact" => {
            team.special_rules.push(SpecialRule::FavouredOfChaos);
        }
        "nurgle" => {
            team.special_rules.push(SpecialRule::FavouredOfNurgle);
        }
        _ => {}
    }

    Ok(team)
}

// ── StarPlayerDef ─────────────────────────────────────────────────────────────

/// A Blood Bowl 2025 star player definition.
///
/// `eligible_races` lists the race names (lower-case) that may hire this star.
/// An empty slice (`&[]`) means the star is available to any team.
#[derive(Clone, Debug)]
pub struct StarPlayerDef {
    pub name: &'static str,
    /// Race names that can hire this star.  Empty = any race.
    pub eligible_races: &'static [&'static str],
    /// Position label used when creating the `Player`.
    pub position: &'static str,
    pub stats: PlayerStats,
    pub skills: &'static [SkillId],
}

// ── Star-player table ─────────────────────────────────────────────────────────

/// Return the `StarPlayerDef` for the given name, or `None` if not found.
pub fn star_player_def(name: &str) -> Option<StarPlayerDef> {
    match name {
        // T-61: Skill-only star players ────────────────────────────────────────
        "Morg 'n' Thorg" => Some(StarPlayerDef {
            name: "Morg 'n' Thorg",
            eligible_races: &[],  // any race
            position: "star_morg_n_thorg",
            stats: PlayerStats::new(6, 6, 4, 11, None),
            skills: &[
                SkillId::Block,
                SkillId::Loner,
                SkillId::MightyBlow,
                SkillId::ThickSkull,
                SkillId::ThrowTeamMate,
            ],
        }),
        "Griff Oberwald" => Some(StarPlayerDef {
            name: "Griff Oberwald",
            eligible_races: &["human", "amazon", "norse"],
            position: "star_griff_oberwald",
            stats: PlayerStats::new(9, 3, 2, 8, None),
            skills: &[
                SkillId::Block,
                SkillId::Dodge,
                SkillId::Sprint,
                SkillId::SureFeet,
                SkillId::Loner,
            ],
        }),
        "Mighty Zug" => Some(StarPlayerDef {
            name: "Mighty Zug",
            eligible_races: &["human", "halfling"],
            position: "star_mighty_zug",
            stats: PlayerStats::new(4, 5, 4, 10, None),
            skills: &[
                SkillId::Block,
                SkillId::MightyBlow,
                SkillId::ThickSkull,
            ],
        }),
        "Deeproot Strongbranch" => Some(StarPlayerDef {
            name: "Deeproot Strongbranch",
            eligible_races: &["halfling", "lizardman", "wood elf"],
            position: "star_deeproot_strongbranch",
            stats: PlayerStats::new(2, 6, 5, 11, None),
            skills: &[
                SkillId::MightyBlow,
                SkillId::StandFirm,
                SkillId::TakeRoot,
                SkillId::ThrowTeamMate,
            ],
        }),

        // T-62: Star players with unique rules ────────────────────────────────
        "Fungus the Loon" => Some(StarPlayerDef {
            name: "Fungus the Loon",
            eligible_races: &["goblin", "orc", "chaos"],
            position: "star_fungus_the_loon",
            stats: PlayerStats::new(4, 7, 4, 9, None),
            skills: &[
                SkillId::BallAndChain,
                SkillId::MightyBlow,
                SkillId::Loner,
                SkillId::Stunty,
            ],
        }),
        "Bomber Dribblesnot" => Some(StarPlayerDef {
            name: "Bomber Dribblesnot",
            eligible_races: &["goblin", "orc"],
            position: "star_bomber_dribblesnot",
            stats: PlayerStats::new(6, 2, 3, 7, None),
            skills: &[
                SkillId::Bombardier,
                SkillId::Dodge,
                SkillId::HailMaryPass,
                SkillId::Loner,
            ],
        }),
        "Helmut Wulf" => Some(StarPlayerDef {
            name: "Helmut Wulf",
            eligible_races: &["human", "amazon", "norse", "orc", "chaos"],
            position: "star_helmut_wulf",
            stats: PlayerStats::new(6, 3, 3, 8, None),
            skills: &[
                SkillId::Chainsaw,
                SkillId::Block,
                SkillId::Loner,
            ],
        }),

        // T-63: Undead/Necromantic star players ───────────────────────────────
        "Count Luthor von Drakenborg" => Some(StarPlayerDef {
            name: "Count Luthor von Drakenborg",
            eligible_races: &["undead", "vampire", "necromantic"],
            position: "star_count_luthor_von_drakenborg",
            stats: PlayerStats::new(6, 4, 2, 9, None),
            skills: &[
                SkillId::HypnoticGaze,
                SkillId::Block,
                SkillId::Regeneration,
            ],
        }),
        "Ramtut III" => Some(StarPlayerDef {
            name: "Ramtut III",
            eligible_races: &["undead", "khemri"],
            position: "star_ramtut_iii",
            stats: PlayerStats::new(5, 5, 4, 10, None),
            skills: &[
                SkillId::Block,
                SkillId::MightyBlow,
                SkillId::Regeneration,
            ],
        }),
        "Setekh" => Some(StarPlayerDef {
            name: "Setekh",
            eligible_races: &["undead", "khemri"],
            position: "star_setekh",
            stats: PlayerStats::new(6, 4, 3, 9, None),
            skills: &[
                SkillId::Block,
                SkillId::MightyBlow,
            ],
        }),

        // T-64: Chaos/Dark Elf star players ───────────────────────────────────
        "Grashnak Blackhoof" => Some(StarPlayerDef {
            name: "Grashnak Blackhoof",
            eligible_races: &["chaos"],
            position: "star_grashnak_blackhoof",
            stats: PlayerStats::new(6, 5, 3, 9, None),
            skills: &[
                SkillId::Horns,
                SkillId::MightyBlow,
                SkillId::Frenzy,
            ],
        }),
        "Lord Borak the Despoiler" => Some(StarPlayerDef {
            name: "Lord Borak the Despoiler",
            eligible_races: &["chaos"],
            position: "star_lord_borak_the_despoiler",
            stats: PlayerStats::new(6, 4, 4, 9, None),
            skills: &[
                SkillId::Block,
                SkillId::Frenzy,
                SkillId::MightyBlow,
                SkillId::Leader,
            ],
        }),
        "Roxanna Darknail" => Some(StarPlayerDef {
            name: "Roxanna Darknail",
            eligible_races: &["dark elf", "high elf", "elf"],
            position: "star_roxanna_darknail",
            stats: PlayerStats::new(7, 3, 2, 8, None),
            skills: &[
                SkillId::Block,
                SkillId::Dodge,
                SkillId::JumpUp,
                SkillId::Stab,
            ],
        }),

        // T-65: Elven star players ─────────────────────────────────────────────
        "Hubris Rakarth" => Some(StarPlayerDef {
            name: "Hubris Rakarth",
            eligible_races: &["dark elf", "high elf"],
            position: "star_hubris_rakarth",
            stats: PlayerStats::new(7, 3, 2, 9, None),
            skills: &[
                SkillId::Block,
                SkillId::Dodge,
                SkillId::Pass,
                SkillId::Tackle,
            ],
        }),
        "Eldril Sidewinder" => Some(StarPlayerDef {
            name: "Eldril Sidewinder",
            eligible_races: &["high elf", "elf", "wood elf"],
            position: "star_eldril_sidewinder",
            stats: PlayerStats::new(7, 3, 2, 8, None),
            skills: &[
                SkillId::Dodge,
                SkillId::HypnoticGaze,
                SkillId::NervesOfSteel,
            ],
        }),
        "Jordell Freshbreeze" => Some(StarPlayerDef {
            name: "Jordell Freshbreeze",
            eligible_races: &["wood elf", "high elf", "elf"],
            position: "star_jordell_freshbreeze",
            stats: PlayerStats::new(9, 3, 2, 8, None),
            skills: &[
                SkillId::Dodge,
                SkillId::Leap,
                SkillId::SideStep,
                SkillId::Sprint,
            ],
        }),

        // T-66: Skaven/Goblin star players ────────────────────────────────────
        "Hakflem Skuttlespike" => Some(StarPlayerDef {
            name: "Hakflem Skuttlespike",
            eligible_races: &["skaven", "underworld"],
            position: "star_hakflem_skuttlespike",
            stats: PlayerStats::new(9, 3, 2, 7, None),
            skills: &[
                SkillId::Dodge,
                SkillId::Leap,
                SkillId::SideStep,
                SkillId::StripBall,
                SkillId::TwoHeads,
            ],
        }),
        "Skitter Stab-Stab" => Some(StarPlayerDef {
            name: "Skitter Stab-Stab",
            eligible_races: &["skaven", "underworld"],
            position: "star_skitter_stab_stab",
            stats: PlayerStats::new(9, 3, 2, 7, None),
            skills: &[
                SkillId::Dodge,
                SkillId::Leap,
                SkillId::Stab,
                SkillId::TwoHeads,
            ],
        }),
        "Nobbla Blackwart" => Some(StarPlayerDef {
            name: "Nobbla Blackwart",
            eligible_races: &["goblin", "orc"],
            position: "star_nobbla_blackwart",
            stats: PlayerStats::new(6, 2, 3, 7, None),
            skills: &[
                SkillId::Block,
                SkillId::Chainsaw,
                SkillId::Dodge,
                SkillId::Loner,
            ],
        }),

        // T-67: Dwarf/Norse/Ogre/Human star players ──────────────────────────
        "Grim Ironjaw" => Some(StarPlayerDef {
            name: "Grim Ironjaw",
            eligible_races: &["dwarf"],
            position: "star_grim_ironjaw",
            stats: PlayerStats::new(5, 3, 3, 10, None),
            skills: &[SkillId::Block, SkillId::MightyBlow, SkillId::ThickSkull],
        }),
        "Barik Farblast" => Some(StarPlayerDef {
            name: "Barik Farblast",
            eligible_races: &["dwarf"],
            position: "star_barik_farblast",
            stats: PlayerStats::new(6, 3, 4, 9, Some(3)),
            skills: &[SkillId::Bombardier, SkillId::HailMaryPass, SkillId::SureHands, SkillId::Loner],
        }),
        "Flint Churnblade" => Some(StarPlayerDef {
            name: "Flint Churnblade",
            eligible_races: &["dwarf"],
            position: "star_flint_churnblade",
            stats: PlayerStats::new(5, 3, 3, 9, None),
            skills: &[SkillId::Chainsaw, SkillId::Dodge, SkillId::Loner],
        }),
        "Brick Far'th" => Some(StarPlayerDef {
            name: "Brick Far'th",
            eligible_races: &["human", "dwarf"],
            position: "star_brick_farth",
            stats: PlayerStats::new(4, 5, 4, 10, None),
            skills: &[SkillId::Block, SkillId::MightyBlow, SkillId::ThickSkull],
        }),

        // T-68: Lizardman/Amazon/Human/misc ──────────────────────────────────
        "Hemlock" => Some(StarPlayerDef {
            name: "Hemlock",
            eligible_races: &["amazon", "lizardman"],
            position: "star_hemlock",
            stats: PlayerStats::new(7, 3, 2, 8, Some(3)),
            skills: &[SkillId::Block, SkillId::NervesOfSteel, SkillId::Pass],
        }),
        "Akhorne the Squirrel" => Some(StarPlayerDef {
            name: "Akhorne the Squirrel",
            eligible_races: &[""],
            position: "star_akhorne",
            stats: PlayerStats::new(7, 1, 2, 6, None),
            skills: &[SkillId::Dodge, SkillId::Frenzy, SkillId::Loner, SkillId::SideStep, SkillId::Stunty],
        }),
        "Dolfar Longstride" => Some(StarPlayerDef {
            name: "Dolfar Longstride",
            eligible_races: &["wood elf", "elf"],
            position: "star_dolfar_longstride",
            stats: PlayerStats::new(8, 3, 2, 8, None),
            skills: &[SkillId::Block, SkillId::Dauntless, SkillId::MightyBlow, SkillId::Sprint],
        }),
        "Quetzal Leap" => Some(StarPlayerDef {
            name: "Quetzal Leap",
            eligible_races: &["lizardman"],
            position: "star_quetzal_leap",
            stats: PlayerStats::new(8, 3, 2, 9, None),
            skills: &[SkillId::Catch, SkillId::DivingCatch, SkillId::Leap],
        }),

        // T-69: Vampire/Undead/Necromantic ────────────────────────────────────
        "Wilhelm Chaney" => Some(StarPlayerDef {
            name: "Wilhelm Chaney",
            eligible_races: &["vampire", "necromantic"],
            position: "star_wilhelm_chaney",
            stats: PlayerStats::new(8, 4, 2, 9, None),
            skills: &[SkillId::Block, SkillId::Claws, SkillId::Frenzy],
        }),
        "Sinnedbad" => Some(StarPlayerDef {
            name: "Sinnedbad",
            eligible_races: &["vampire"],
            position: "star_sinnedbad",
            stats: PlayerStats::new(6, 3, 2, 8, None),
            skills: &[SkillId::HypnoticGaze],
        }),
        "Zolcath the Zoat" => Some(StarPlayerDef {
            name: "Zolcath the Zoat",
            eligible_races: &["lizardman"],
            position: "star_zolcath",
            stats: PlayerStats::new(6, 5, 3, 10, None),
            skills: &[SkillId::Block, SkillId::JumpUp, SkillId::MightyBlow, SkillId::Tentacles],
        }),
        "J. Earlice" => Some(StarPlayerDef {
            name: "J. Earlice",
            eligible_races: &["undead", "necromantic"],
            position: "star_j_earlice",
            stats: PlayerStats::new(6, 3, 2, 8, Some(3)),
            skills: &[SkillId::Block, SkillId::Pass],
        }),

        // T-70: Chaos Pact, Underworld, multi-team ────────────────────────────
        "Max Spleenripper" => Some(StarPlayerDef {
            name: "Max Spleenripper",
            eligible_races: &["chaos pact", "chaos"],
            position: "star_max_spleenripper",
            stats: PlayerStats::new(6, 4, 3, 9, None),
            skills: &[SkillId::Block, SkillId::MightyBlow, SkillId::PrehensileTail, SkillId::Tentacles],
        }),
        "Varag Ghoul-Chewer" => Some(StarPlayerDef {
            name: "Varag Ghoul-Chewer",
            eligible_races: &["orc"],
            position: "star_varag",
            stats: PlayerStats::new(6, 5, 3, 10, None),
            skills: &[SkillId::Block, SkillId::Frenzy, SkillId::MightyBlow, SkillId::Juggernaut],
        }),
        "Rashnak Backstabber" => Some(StarPlayerDef {
            name: "Rashnak Backstabber",
            eligible_races: &["goblin", "orc", "underworld"],
            position: "star_rashnak",
            stats: PlayerStats::new(6, 2, 3, 7, None),
            skills: &[SkillId::DirtyPlayer, SkillId::Shadowing, SkillId::SneakyGit],
        }),
        "Skrull Halfheight" => Some(StarPlayerDef {
            name: "Skrull Halfheight",
            eligible_races: &["halfling", "goblin"],
            position: "star_skrull",
            stats: PlayerStats::new(6, 2, 3, 6, None),
            skills: &[SkillId::Dodge, SkillId::SideStep, SkillId::Stunty],
        }),

        // T-71: Khemri specific ───────────────────────────────────────────────
        "Hthark the Unstoppable" => Some(StarPlayerDef {
            name: "Hthark the Unstoppable",
            eligible_races: &["khemri", "undead"],
            position: "star_hthark",
            stats: PlayerStats::new(6, 5, 4, 10, None),
            skills: &[SkillId::Block, SkillId::BreakTackle, SkillId::MightyBlow],
        }),
        "Valen Swift" => Some(StarPlayerDef {
            name: "Valen Swift",
            eligible_races: &["khemri", "undead"],
            position: "star_valen_swift",
            stats: PlayerStats::new(8, 3, 3, 8, None),
            skills: &[SkillId::Block, SkillId::Dodge, SkillId::Sprint, SkillId::SureFeet],
        }),
        "Headsplitter" => Some(StarPlayerDef {
            name: "Headsplitter",
            eligible_races: &["khemri"],
            position: "star_headsplitter",
            stats: PlayerStats::new(6, 4, 4, 9, None),
            skills: &[SkillId::Block, SkillId::MightyBlow],
        }),
        "Josef Bugman" => Some(StarPlayerDef {
            name: "Josef Bugman",
            eligible_races: &["dwarf"],
            position: "star_josef_bugman",
            stats: PlayerStats::new(5, 3, 3, 10, None),
            skills: &[SkillId::Block, SkillId::Dauntless, SkillId::ThickSkull],
        }),

        // T-72: Remaining unique star players ─────────────────────────────────
        "Grotty" => Some(StarPlayerDef {
            name: "Grotty",
            eligible_races: &["goblin"],
            position: "star_grotty",
            stats: PlayerStats::new(5, 2, 3, 6, None),
            skills: &[SkillId::DirtyPlayer, SkillId::Dodge, SkillId::RightStuff, SkillId::Stunty],
        }),
        "Fezglitch" => Some(StarPlayerDef {
            name: "Fezglitch",
            eligible_races: &["goblin"],
            position: "star_fezglitch",
            stats: PlayerStats::new(4, 7, 4, 9, None),
            skills: &[SkillId::BallAndChain, SkillId::MightyBlow, SkillId::Loner],
        }),
        "Lucien Swift" => Some(StarPlayerDef {
            name: "Lucien Swift",
            eligible_races: &["high elf", "elf", "wood elf"],
            position: "star_lucien_swift",
            stats: PlayerStats::new(8, 3, 2, 8, None),
            skills: &[SkillId::Block, SkillId::Dodge, SkillId::Tackle],
        }),
        "Prince Moranion" => Some(StarPlayerDef {
            name: "Prince Moranion",
            eligible_races: &["wood elf", "high elf"],
            position: "star_prince_moranion",
            stats: PlayerStats::new(8, 3, 2, 8, None),
            skills: &[SkillId::Block, SkillId::Dodge, SkillId::Leap],
        }),
        "Lewdgrip Whiparm" => Some(StarPlayerDef {
            name: "Lewdgrip Whiparm",
            eligible_races: &["chaos", "chaos pact"],
            position: "star_lewdgrip",
            stats: PlayerStats::new(5, 3, 3, 9, None),
            skills: &[SkillId::PrehensileTail, SkillId::Tentacles],
        }),

        // T-73: Final star players ─────────────────────────────────────────────
        "Bertha Bigfist" => Some(StarPlayerDef {
            name: "Bertha Bigfist",
            eligible_races: &["ogre"],
            position: "star_bertha_bigfist",
            stats: PlayerStats::new(5, 5, 4, 10, None),
            skills: &[SkillId::Block, SkillId::MightyBlow, SkillId::ThickSkull],
        }),
        "Boomer Eziasson" => Some(StarPlayerDef {
            name: "Boomer Eziasson",
            eligible_races: &["norse"],
            position: "star_boomer_eziasson",
            stats: PlayerStats::new(6, 3, 3, 8, None),
            skills: &[SkillId::Block, SkillId::Frenzy, SkillId::MightyBlow],
        }),
        "Icepelt Hammerblow" => Some(StarPlayerDef {
            name: "Icepelt Hammerblow",
            eligible_races: &["norse"],
            position: "star_icepelt",
            stats: PlayerStats::new(6, 4, 3, 9, None),
            skills: &[SkillId::Block, SkillId::MightyBlow, SkillId::Juggernaut],
        }),
        "Ripper" => Some(StarPlayerDef {
            name: "Ripper",
            eligible_races: &["orc"],
            position: "star_ripper",
            stats: PlayerStats::new(5, 6, 5, 10, None),
            skills: &[SkillId::Block, SkillId::Loner, SkillId::MightyBlow, SkillId::ThrowTeamMate],
        }),
        "Scrappa Sorehead" => Some(StarPlayerDef {
            name: "Scrappa Sorehead",
            eligible_races: &["goblin", "orc"],
            position: "star_scrappa",
            stats: PlayerStats::new(7, 2, 3, 7, None),
            skills: &[SkillId::Dodge, SkillId::Loner, SkillId::SideStep, SkillId::Stunty, SkillId::SureFeet],
        }),
        "Soaren Hightower" => Some(StarPlayerDef {
            name: "Soaren Hightower",
            eligible_races: &["high elf", "elf"],
            position: "star_soaren",
            stats: PlayerStats::new(8, 3, 2, 8, Some(3)),
            skills: &[SkillId::Block, SkillId::Dodge, SkillId::Pass, SkillId::SureFeet],
        }),
        "Ugroth Bolgrot" => Some(StarPlayerDef {
            name: "Ugroth Bolgrot",
            eligible_races: &["orc", "chaos"],
            position: "star_ugroth",
            stats: PlayerStats::new(5, 4, 3, 9, None),
            skills: &[SkillId::Chainsaw, SkillId::Loner],
        }),
        "Zzharg Madeye" => Some(StarPlayerDef {
            name: "Zzharg Madeye",
            eligible_races: &["chaos dwarf"],
            position: "star_zzharg",
            stats: PlayerStats::new(5, 4, 3, 9, None),
            skills: &[SkillId::Block, SkillId::MightyBlow, SkillId::ThickSkull],
        }),

        _ => None,
    }
}

// ── make_star_player ──────────────────────────────────────────────────────────

/// Create a `Player` from a star player definition.
///
/// Returns `Err` if no star with that name exists.
pub fn make_star_player(name: &str, team_id: TeamId) -> Result<Player, String> {
    let def = star_player_def(name).ok_or_else(|| format!("Unknown star player: {name}"))?;
    let player_id = PlayerId(format!("star_{}", def.position));
    let skills: SkillSet = def.skills.iter().copied().collect();
    Ok(Player::new(
        player_id,
        def.name.to_string(),
        def.position.to_string(),
        team_id,
        0, // star players don't use jersey numbers in the standard sense
        def.stats,
        skills,
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_core::types::TeamId;

    #[test]
    fn make_human_team_correct_stats() {
        let team = make_team("human", TeamId::Home, "h", "Reavers", 3).unwrap();
        let blitzer = team.players().iter().find(|p| p.position_id.contains("litzer"));
        assert!(blitzer.is_some(), "no blitzer found");
        assert!(
            blitzer.unwrap().has_skill(SkillId::Block),
            "blitzer must have Block"
        );
    }

    #[test]
    fn make_human_team_has_11_players() {
        let team = make_team("human", TeamId::Home, "h", "Reavers", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_orc_team_has_11_players() {
        let team = make_team("orc", TeamId::Away, "a", "Grudgebearers", 2).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_dark_elf_team() {
        let team = make_team("dark elf", TeamId::Home, "de", "Dark Elves", 3).unwrap();
        assert!(team.players().len() <= 11);
        let witch = team.players().iter().find(|p| p.position_id.contains("witch"));
        assert!(witch.is_some());
        assert!(witch.unwrap().has_skill(SkillId::Frenzy));
    }

    #[test]
    fn make_skaven_team() {
        let team = make_team("skaven", TeamId::Home, "sk", "Skavenpool", 3).unwrap();
        let gr = team.players().iter().find(|p| p.position_id.contains("gutter"));
        assert!(gr.is_some());
        assert!(gr.unwrap().has_skill(SkillId::Dodge));
    }

    #[test]
    fn make_undead_team() {
        let team = make_team("undead", TeamId::Home, "ud", "Tomb Kings", 2).unwrap();
        let mummy = team.players().iter().find(|p| p.position_id.contains("mummy"));
        assert!(mummy.is_some());
        assert!(mummy.unwrap().has_skill(SkillId::MightyBlow));
    }

    #[test]
    fn make_dwarf_team() {
        let team = make_team("dwarf", TeamId::Home, "dw", "Dwarfs", 3).unwrap();
        let slayer = team.players().iter().find(|p| p.position_id.contains("troll_slayer"));
        assert!(slayer.is_some());
        assert!(slayer.unwrap().has_skill(SkillId::Block));
        assert!(slayer.unwrap().has_skill(SkillId::Frenzy));
    }

    #[test]
    fn unknown_race_returns_error() {
        assert!(make_team("atlantean", TeamId::Home, "x", "Unknown", 1).is_err());
    }

    #[test]
    fn jersey_numbers_are_sequential() {
        let team = make_team("human", TeamId::Home, "h", "Reavers", 3).unwrap();
        let jerseys: Vec<u8> = team.players().iter().map(|p| p.jersey_number).collect();
        for (i, &j) in jerseys.iter().enumerate() {
            assert_eq!(j, (i + 1) as u8, "jersey number should be sequential");
        }
    }

    // ── star player tests ─────────────────────────────────────────────────────

    #[test]
    fn star_morg_n_thorg() {
        let def = star_player_def("Morg 'n' Thorg").unwrap();
        assert_eq!(def.stats.ma, 6);
        assert_eq!(def.stats.st, 6);
        assert_eq!(def.eligible_races.len(), 0, "Morg should be available to any race");
        assert!(def.skills.contains(&SkillId::Block));
        assert!(def.skills.contains(&SkillId::ThrowTeamMate));
    }

    #[test]
    fn star_griff_oberwald() {
        let def = star_player_def("Griff Oberwald").unwrap();
        assert_eq!(def.stats.ma, 9);
        assert_eq!(def.stats.st, 3);
        assert!(def.skills.contains(&SkillId::Sprint));
        assert!(def.eligible_races.contains(&"human"));
    }

    #[test]
    fn star_mighty_zug() {
        let def = star_player_def("Mighty Zug").unwrap();
        assert_eq!(def.stats.st, 5);
        assert_eq!(def.stats.av, 10);
        assert!(def.skills.contains(&SkillId::MightyBlow));
    }

    #[test]
    fn star_deeproot_strongbranch() {
        let def = star_player_def("Deeproot Strongbranch").unwrap();
        assert_eq!(def.stats.ma, 2);
        assert_eq!(def.stats.st, 6);
        assert!(def.skills.contains(&SkillId::TakeRoot));
        assert!(def.skills.contains(&SkillId::ThrowTeamMate));
    }

    #[test]
    fn star_fungus_the_loon() {
        let def = star_player_def("Fungus the Loon").unwrap();
        assert_eq!(def.stats.st, 7);
        assert!(def.skills.contains(&SkillId::BallAndChain));
        assert!(def.skills.contains(&SkillId::Stunty));
    }

    #[test]
    fn star_bomber_dribblesnot() {
        let def = star_player_def("Bomber Dribblesnot").unwrap();
        assert_eq!(def.stats.av, 7);
        assert!(def.skills.contains(&SkillId::Bombardier));
        assert!(def.skills.contains(&SkillId::HailMaryPass));
    }

    #[test]
    fn star_helmut_wulf() {
        let def = star_player_def("Helmut Wulf").unwrap();
        assert_eq!(def.stats.ma, 6);
        assert!(def.skills.contains(&SkillId::Chainsaw));
        assert!(def.eligible_races.contains(&"orc"));
    }

    #[test]
    fn star_count_luthor_von_drakenborg() {
        let def = star_player_def("Count Luthor von Drakenborg").unwrap();
        assert_eq!(def.stats.st, 4);
        assert!(def.skills.contains(&SkillId::HypnoticGaze));
        assert!(def.skills.contains(&SkillId::Regeneration));
    }

    #[test]
    fn star_ramtut_iii() {
        let def = star_player_def("Ramtut III").unwrap();
        assert_eq!(def.stats.av, 10);
        assert!(def.skills.contains(&SkillId::Block));
        assert!(def.skills.contains(&SkillId::Regeneration));
    }

    #[test]
    fn star_setekh() {
        let def = star_player_def("Setekh").unwrap();
        assert_eq!(def.stats.ma, 6);
        assert_eq!(def.stats.st, 4);
        assert!(def.skills.contains(&SkillId::Block));
        assert!(def.skills.contains(&SkillId::MightyBlow));
    }

    #[test]
    fn star_grashnak_blackhoof() {
        let def = star_player_def("Grashnak Blackhoof").unwrap();
        assert_eq!(def.stats.st, 5);
        assert!(def.skills.contains(&SkillId::Horns));
        assert!(def.skills.contains(&SkillId::Frenzy));
    }

    #[test]
    fn star_lord_borak_the_despoiler() {
        let def = star_player_def("Lord Borak the Despoiler").unwrap();
        assert_eq!(def.stats.av, 9);
        assert!(def.skills.contains(&SkillId::Leader));
        assert!(def.skills.contains(&SkillId::Frenzy));
    }

    #[test]
    fn star_roxanna_darknail() {
        let def = star_player_def("Roxanna Darknail").unwrap();
        assert_eq!(def.stats.ma, 7);
        assert!(def.skills.contains(&SkillId::Stab));
        assert!(def.skills.contains(&SkillId::JumpUp));
    }

    #[test]
    fn star_hubris_rakarth() {
        let def = star_player_def("Hubris Rakarth").unwrap();
        assert_eq!(def.stats.av, 9);
        assert!(def.skills.contains(&SkillId::Tackle));
        assert!(def.skills.contains(&SkillId::Pass));
    }

    #[test]
    fn star_eldril_sidewinder() {
        let def = star_player_def("Eldril Sidewinder").unwrap();
        assert_eq!(def.stats.ma, 7);
        assert!(def.skills.contains(&SkillId::HypnoticGaze));
        assert!(def.skills.contains(&SkillId::NervesOfSteel));
    }

    #[test]
    fn star_jordell_freshbreeze() {
        let def = star_player_def("Jordell Freshbreeze").unwrap();
        assert_eq!(def.stats.ma, 9);
        assert!(def.skills.contains(&SkillId::Leap));
        assert!(def.skills.contains(&SkillId::SideStep));
    }

    #[test]
    fn star_hakflem_skuttlespike() {
        let def = star_player_def("Hakflem Skuttlespike").unwrap();
        assert_eq!(def.stats.av, 7);
        assert!(def.skills.contains(&SkillId::TwoHeads));
        assert!(def.skills.contains(&SkillId::StripBall));
    }

    #[test]
    fn star_skitter_stab_stab() {
        let def = star_player_def("Skitter Stab-Stab").unwrap();
        assert_eq!(def.stats.ma, 9);
        assert!(def.skills.contains(&SkillId::Stab));
        assert!(def.skills.contains(&SkillId::TwoHeads));
    }

    #[test]
    fn star_nobbla_blackwart() {
        let def = star_player_def("Nobbla Blackwart").unwrap();
        assert_eq!(def.stats.av, 7);
        assert!(def.skills.contains(&SkillId::Chainsaw));
        assert!(def.skills.contains(&SkillId::Dodge));
    }

    #[test]
    fn star_unknown_returns_none() {
        assert!(star_player_def("Nobody McFakerson").is_none());
    }

    #[test]
    fn make_star_player_creates_player() {
        let player = make_star_player("Morg 'n' Thorg", TeamId::Home).unwrap();
        assert!(player.has_skill(SkillId::Block));
        assert!(player.has_skill(SkillId::MightyBlow));
        assert!(player.has_skill(SkillId::ThrowTeamMate));
    }

    #[test]
    fn make_star_player_unknown_returns_err() {
        assert!(make_star_player("Nobody McFakerson", TeamId::Home).is_err());
    }

    // ── T-59 new race tests ────────────────────────────────────────────────

    #[test]
    fn make_amazon_team_has_11_players() {
        let team = make_team("amazon", TeamId::Home, "az", "Amazons", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_amazon_team_blitzer_has_block_dodge() {
        let team = make_team("amazon", TeamId::Home, "az", "Amazons", 3).unwrap();
        let blitzer = team.players().iter().find(|p| p.position_id.contains("blitzer")).unwrap();
        assert!(blitzer.has_skill(SkillId::Block));
        assert!(blitzer.has_skill(SkillId::Dodge));
    }

    #[test]
    fn make_chaos_team_has_11_players() {
        let team = make_team("chaos", TeamId::Home, "ch", "Chaos All-Stars", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_chaos_team_minotaur_has_frenzy() {
        let team = make_team("chaos", TeamId::Home, "ch", "Chaos All-Stars", 3).unwrap();
        let minotaur = team.players().iter().find(|p| p.position_id.contains("minotaur")).unwrap();
        assert!(minotaur.has_skill(SkillId::Frenzy));
        assert!(minotaur.has_skill(SkillId::MightyBlow));
    }

    #[test]
    fn make_norse_team_has_11_players() {
        let team = make_team("norse", TeamId::Home, "no", "Norse Warriors", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_norse_team_berserker_has_block_frenzy() {
        let team = make_team("norse", TeamId::Home, "no", "Norse Warriors", 3).unwrap();
        let berserker = team.players().iter().find(|p| p.position_id.contains("berserker")).unwrap();
        assert!(berserker.has_skill(SkillId::Block));
        assert!(berserker.has_skill(SkillId::Frenzy));
    }

    #[test]
    fn make_halfling_team_has_11_players() {
        let team = make_team("halfling", TeamId::Home, "hf", "Greenfield Grasshuggers", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_halfling_team_treeman_has_mighty_blow() {
        let team = make_team("halfling", TeamId::Home, "hf", "Greenfield Grasshuggers", 3).unwrap();
        let treeman = team.players().iter().find(|p| p.position_id.contains("treeman")).unwrap();
        assert!(treeman.has_skill(SkillId::MightyBlow));
        assert!(treeman.has_skill(SkillId::ThrowTeamMate));
    }

    #[test]
    fn make_high_elf_team_has_11_players() {
        let team = make_team("high elf", TeamId::Home, "he", "High Elves", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_high_elf_team_blitzer_has_side_step() {
        let team = make_team("high elf", TeamId::Home, "he", "High Elves", 3).unwrap();
        let blitzer = team.players().iter().find(|p| p.position_id.contains("blitzer")).unwrap();
        assert!(blitzer.has_skill(SkillId::Block));
        assert!(blitzer.has_skill(SkillId::SideStep));
    }

    // T-67
    #[test]
    fn star_grim_ironjaw() {
        let def = star_player_def("Grim Ironjaw").unwrap();
        assert!(def.skills.contains(&SkillId::MightyBlow));
        assert!(def.skills.contains(&SkillId::ThickSkull));
    }
    #[test]
    fn star_barik_farblast() {
        let def = star_player_def("Barik Farblast").unwrap();
        assert!(def.skills.contains(&SkillId::Bombardier));
        assert!(def.stats.pa.is_some());
    }
    #[test]
    fn star_flint_churnblade() {
        let def = star_player_def("Flint Churnblade").unwrap();
        assert!(def.skills.contains(&SkillId::Chainsaw));
    }
    #[test]
    fn star_brick_farth() {
        let def = star_player_def("Brick Far'th").unwrap();
        assert_eq!(def.stats.st, 5);
        assert!(def.skills.contains(&SkillId::Block));
    }

    // T-68
    #[test]
    fn star_hemlock() {
        let def = star_player_def("Hemlock").unwrap();
        assert!(def.skills.contains(&SkillId::Pass));
    }
    #[test]
    fn star_akhorne_the_squirrel() {
        let def = star_player_def("Akhorne the Squirrel").unwrap();
        assert!(def.skills.contains(&SkillId::Frenzy));
        assert!(def.skills.contains(&SkillId::Stunty));
    }
    #[test]
    fn star_dolfar_longstride() {
        let def = star_player_def("Dolfar Longstride").unwrap();
        assert_eq!(def.stats.ma, 8);
        assert!(def.skills.contains(&SkillId::Sprint));
    }
    #[test]
    fn star_quetzal_leap() {
        let def = star_player_def("Quetzal Leap").unwrap();
        assert!(def.skills.contains(&SkillId::Catch));
        assert!(def.skills.contains(&SkillId::Leap));
    }

    // T-69
    #[test]
    fn star_wilhelm_chaney() {
        let def = star_player_def("Wilhelm Chaney").unwrap();
        assert_eq!(def.stats.ma, 8);
        assert!(def.skills.contains(&SkillId::Claws));
    }
    #[test]
    fn star_sinnedbad() {
        let def = star_player_def("Sinnedbad").unwrap();
        assert!(def.skills.contains(&SkillId::HypnoticGaze));
    }
    #[test]
    fn star_zolcath_the_zoat() {
        let def = star_player_def("Zolcath the Zoat").unwrap();
        assert_eq!(def.stats.st, 5);
        assert!(def.skills.contains(&SkillId::Tentacles));
    }
    #[test]
    fn star_j_earlice() {
        let def = star_player_def("J. Earlice").unwrap();
        assert!(def.skills.contains(&SkillId::Pass));
    }

    // T-70
    #[test]
    fn star_max_spleenripper() {
        let def = star_player_def("Max Spleenripper").unwrap();
        assert!(def.skills.contains(&SkillId::Tentacles));
        assert!(def.skills.contains(&SkillId::PrehensileTail));
    }
    #[test]
    fn star_varag_ghoul_chewer() {
        let def = star_player_def("Varag Ghoul-Chewer").unwrap();
        assert_eq!(def.stats.st, 5);
        assert!(def.skills.contains(&SkillId::Juggernaut));
    }
    #[test]
    fn star_rashnak_backstabber() {
        let def = star_player_def("Rashnak Backstabber").unwrap();
        assert!(def.skills.contains(&SkillId::Shadowing));
    }
    #[test]
    fn star_skrull_halfheight() {
        let def = star_player_def("Skrull Halfheight").unwrap();
        assert!(def.skills.contains(&SkillId::Stunty));
        assert!(def.skills.contains(&SkillId::SideStep));
    }

    // T-71
    #[test]
    fn star_hthark_the_unstoppable() {
        let def = star_player_def("Hthark the Unstoppable").unwrap();
        assert_eq!(def.stats.st, 5);
        assert!(def.skills.contains(&SkillId::BreakTackle));
    }
    #[test]
    fn star_valen_swift() {
        let def = star_player_def("Valen Swift").unwrap();
        assert_eq!(def.stats.ma, 8);
        assert!(def.skills.contains(&SkillId::Sprint));
    }
    #[test]
    fn star_headsplitter() {
        let def = star_player_def("Headsplitter").unwrap();
        assert!(def.skills.contains(&SkillId::MightyBlow));
    }
    #[test]
    fn star_josef_bugman() {
        let def = star_player_def("Josef Bugman").unwrap();
        assert!(def.skills.contains(&SkillId::Dauntless));
        assert!(def.skills.contains(&SkillId::ThickSkull));
    }

    // T-72
    #[test]
    fn star_grotty() {
        let def = star_player_def("Grotty").unwrap();
        assert!(def.skills.contains(&SkillId::RightStuff));
        assert!(def.skills.contains(&SkillId::Stunty));
    }
    #[test]
    fn star_fezglitch() {
        let def = star_player_def("Fezglitch").unwrap();
        assert!(def.skills.contains(&SkillId::BallAndChain));
        assert_eq!(def.stats.st, 7);
    }
    #[test]
    fn star_lucien_swift() {
        let def = star_player_def("Lucien Swift").unwrap();
        assert!(def.skills.contains(&SkillId::Block));
        assert_eq!(def.stats.ma, 8);
    }
    #[test]
    fn star_prince_moranion() {
        let def = star_player_def("Prince Moranion").unwrap();
        assert!(def.skills.contains(&SkillId::Leap));
    }
    #[test]
    fn star_lewdgrip_whiparm() {
        let def = star_player_def("Lewdgrip Whiparm").unwrap();
        assert!(def.skills.contains(&SkillId::PrehensileTail));
        assert!(def.skills.contains(&SkillId::Tentacles));
    }

    // T-73
    #[test]
    fn star_bertha_bigfist() {
        let def = star_player_def("Bertha Bigfist").unwrap();
        assert_eq!(def.stats.st, 5);
        assert!(def.skills.contains(&SkillId::MightyBlow));
    }
    #[test]
    fn star_boomer_eziasson() {
        let def = star_player_def("Boomer Eziasson").unwrap();
        assert!(def.skills.contains(&SkillId::Frenzy));
    }
    #[test]
    fn star_icepelt_hammerblow() {
        let def = star_player_def("Icepelt Hammerblow").unwrap();
        assert_eq!(def.stats.st, 4);
        assert!(def.skills.contains(&SkillId::Juggernaut));
    }
    #[test]
    fn star_ripper() {
        let def = star_player_def("Ripper").unwrap();
        assert_eq!(def.stats.st, 6);
        assert!(def.skills.contains(&SkillId::ThrowTeamMate));
    }
    #[test]
    fn star_scrappa_sorehead() {
        let def = star_player_def("Scrappa Sorehead").unwrap();
        assert!(def.skills.contains(&SkillId::Stunty));
        assert!(def.skills.contains(&SkillId::SureFeet));
    }
    #[test]
    fn star_ugroth_bolgrot() {
        let def = star_player_def("Ugroth Bolgrot").unwrap();
        assert!(def.skills.contains(&SkillId::Chainsaw));
    }
    #[test]
    fn star_soaren_hightower() {
        let def = star_player_def("Soaren Hightower").unwrap();
        assert!(def.skills.contains(&SkillId::Pass));
        assert_eq!(def.stats.ma, 8);
    }
    #[test]
    fn star_zzharg_madeye() {
        let def = star_player_def("Zzharg Madeye").unwrap();
        assert!(def.skills.contains(&SkillId::ThickSkull));
    }

    // ── T-59 additional race tests ─────────────────────────────────────────

    #[test]
    fn make_goblin_team_has_players() {
        let team = make_team("goblin", TeamId::Home, "go", "Gobbo Gang", 3).unwrap();
        assert!(team.players().len() >= 2);
        let goblin = team.players().iter().find(|p| p.position_id.contains("goblin_goblin")).unwrap();
        assert!(goblin.has_skill(SkillId::Stunty));
        assert!(goblin.has_skill(SkillId::Dodge));
    }

    #[test]
    fn make_lizardman_team_has_saurus_and_skinks() {
        let team = make_team("lizardman", TeamId::Home, "lz", "Lizardmen", 3).unwrap();
        assert!(team.players().len() >= 2);
        let skink = team.players().iter().find(|p| p.position_id.contains("skink")).unwrap();
        assert!(skink.has_skill(SkillId::Dodge));
        assert!(skink.has_skill(SkillId::Stunty));
    }

    #[test]
    fn make_elf_team_has_11_players() {
        let team = make_team("elf", TeamId::Home, "el", "Pro Elves", 3).unwrap();
        assert_eq!(team.players().len(), 11);
    }

    #[test]
    fn make_wood_elf_team_wardancer_has_leap() {
        let team = make_team("wood elf", TeamId::Home, "we", "Wood Elves", 3).unwrap();
        let wardancer = team.players().iter().find(|p| p.position_id.contains("wardancer")).unwrap();
        assert!(wardancer.has_skill(SkillId::Leap));
        assert!(wardancer.has_skill(SkillId::Block));
    }

    #[test]
    fn make_chaos_dwarf_team_has_bull_centaur() {
        let team = make_team("chaos dwarf", TeamId::Home, "cd", "Chaos Dwarfs", 3).unwrap();
        let centaur = team.players().iter().find(|p| p.position_id.contains("bull_centaur")).unwrap();
        assert!(centaur.has_skill(SkillId::Sprint));
        assert!(centaur.has_skill(SkillId::SureFeet));
    }

    #[test]
    fn make_necromantic_team_has_werewolf() {
        let team = make_team("necromantic", TeamId::Home, "ne", "Necromantic Horror", 3).unwrap();
        let werewolf = team.players().iter().find(|p| p.position_id.contains("werewolf")).unwrap();
        assert!(werewolf.has_skill(SkillId::Claws));
        assert!(werewolf.has_skill(SkillId::Frenzy));
    }

    #[test]
    fn make_nurgle_team_has_beast_of_nurgle() {
        let team = make_team("nurgle", TeamId::Home, "nu", "Nurgle Rotters", 3).unwrap();
        let beast = team.players().iter().find(|p| p.position_id.contains("beast")).unwrap();
        assert!(beast.has_skill(SkillId::Tentacles));
        assert!(beast.has_skill(SkillId::Regeneration));
    }

    #[test]
    fn make_vampire_team_has_vampires() {
        let team = make_team("vampire", TeamId::Home, "vp", "Vampire Counts", 3).unwrap();
        let vampire = team.players().iter().find(|p| p.position_id.contains("vampire_vampire")).unwrap();
        assert!(vampire.has_skill(SkillId::Bloodlust));
        assert!(vampire.has_skill(SkillId::Regeneration));
    }

    #[test]
    fn make_khemri_team_has_tomb_guardians() {
        let team = make_team("khemri", TeamId::Home, "kh", "Khemri Tomb Kings", 3).unwrap();
        let guardian = team.players().iter().find(|p| p.position_id.contains("tomb_guardian")).unwrap();
        assert!(guardian.has_skill(SkillId::MightyBlow));
        assert!(guardian.has_skill(SkillId::Regeneration));
    }

    #[test]
    fn make_ogre_team_has_ogres() {
        let team = make_team("ogre", TeamId::Home, "og", "Ogre Kingdoms", 3).unwrap();
        let ogre = team.players().iter().find(|p| p.position_id.contains("ogre_ogre")).unwrap();
        assert!(ogre.has_skill(SkillId::MightyBlow));
        assert!(ogre.has_skill(SkillId::ThrowTeamMate));
    }

    #[test]
    fn make_slann_team_wardancer_has_leap() {
        let team = make_team("slann", TeamId::Home, "sl", "Slann Starmaster", 3).unwrap();
        let blitzer = team.players().iter().find(|p| p.position_id.contains("blitzer")).unwrap();
        assert!(blitzer.has_skill(SkillId::Leap));
        assert!(blitzer.has_skill(SkillId::Block));
    }

    #[test]
    fn make_underworld_team_has_troll() {
        let team = make_team("underworld", TeamId::Home, "uw", "Underworld Creepers", 3).unwrap();
        let troll = team.players().iter().find(|p| p.position_id.contains("troll")).unwrap();
        assert!(troll.has_skill(SkillId::AlwaysHungry));
        assert!(troll.has_skill(SkillId::ThrowTeamMate));
    }

    #[test]
    fn necromantic_team_has_masters_of_undeath_special_rule() {
        use ffb_core::types::SpecialRule;
        let team = make_team("necromantic", TeamId::Home, "ne", "Necromantic Horror", 3).unwrap();
        assert!(team.has_special_rule(SpecialRule::MastersOfUndeath));
    }

    #[test]
    fn goblin_team_has_swarming_special_rule() {
        use ffb_core::types::SpecialRule;
        let team = make_team("goblin", TeamId::Home, "go", "Gobbo Gang", 3).unwrap();
        assert!(team.has_special_rule(SpecialRule::Swarming));
    }
}
