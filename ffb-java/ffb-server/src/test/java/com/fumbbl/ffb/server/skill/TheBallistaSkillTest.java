package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2020.special.TheBallista;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TheBallistaSkillTest {

    private TheBallista skill;

    @BeforeEach
    void setUp() {
        skill = new TheBallista();
        skill.postConstruct();
    }

    @Test
    void name_is_The_Ballista() {
        assertEquals("The Ballista", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = TheBallista.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }

    @Test
    void has_the_ballista_reroll_sources() {
        assertNotNull(skill.getRerollSource(ReRolledActions.PASS),
            "The Ballista (bb2020) must register a reroll source for pass rolls");
        assertNotNull(skill.getRerollSource(ReRolledActions.THROW_TEAM_MATE),
            "The Ballista (bb2020) must register a reroll source for throw team-mate rolls");
    }
}
