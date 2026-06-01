package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.StrengthIncrease;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StrengthIncreaseSkillTest {

    private StrengthIncrease skill;

    @BeforeEach
    void setUp() {
        skill = new StrengthIncrease();
        skill.postConstruct();
    }

    @Test
    void name_is_plus_ST() {
        assertEquals("+ST", skill.getName());
    }

    @Test
    void category_is_stat_increase() {
        assertEquals(SkillCategory.STAT_INCREASE, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = StrengthIncrease.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
