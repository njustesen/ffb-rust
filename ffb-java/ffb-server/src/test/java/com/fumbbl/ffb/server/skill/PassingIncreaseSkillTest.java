package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2020.PassingIncrease;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PassingIncreaseSkillTest {

    private PassingIncrease skill;

    @BeforeEach
    void setUp() {
        skill = new PassingIncrease();
        skill.postConstruct();
    }

    @Test
    void name_is_plus_PA() {
        assertEquals("+PA", skill.getName());
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
    void is_bb2020_edition() {
        RulesCollection annotation = PassingIncrease.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
