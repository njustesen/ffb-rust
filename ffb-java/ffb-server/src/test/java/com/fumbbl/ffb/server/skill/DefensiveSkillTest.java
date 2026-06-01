package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2020.Defensive;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DefensiveSkillTest {

    private Defensive skill;

    @BeforeEach
    void setUp() {
        skill = new Defensive();
        skill.postConstruct();
    }

    @Test
    void name_is_Defensive() {
        assertEquals("Defensive", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = Defensive.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
