//! Translation of `com.fumbbl.ffb.Constant` (the parts the engine reads).
use crate::enums::SkillId;

/// Java `Constant.getGrantAbleSkills(skillFactory)`: the skills Wisdom of the White Dwarf may grant,
/// with their configured value (`GRANT_ABLE_SKILLS`: Break Tackle, Dauntless, Mighty Blow "1",
/// Sure Feet). Java maps each name through the ruleset's SkillFactory and drops unknown ones; all
/// four exist in BB2020 and BB2025, the only rulesets with a granting skill. Java collects into a
/// HashSet; every caller here only asks "is any of them missing", so the order is irrelevant.
pub fn grant_able_skills() -> Vec<(SkillId, Option<&'static str>)> {
    vec![
        (SkillId::BreakTackle, None),
        (SkillId::Dauntless, None),
        (SkillId::MightyBlow, Some("1")),
        (SkillId::SureFeet, None),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_able_skills_are_javas_four() {
        let skills: Vec<SkillId> = grant_able_skills().into_iter().map(|(s, _)| s).collect();
        assert_eq!(skills, vec![SkillId::BreakTackle, SkillId::Dauntless, SkillId::MightyBlow, SkillId::SureFeet]);
        assert_eq!(grant_able_skills()[2].1, Some("1"));
    }
}
