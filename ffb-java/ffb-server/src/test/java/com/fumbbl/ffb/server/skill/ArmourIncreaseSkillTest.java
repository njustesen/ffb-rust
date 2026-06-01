package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.ArmourIncrease;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ArmourIncreaseSkillTest {

    private ArmourIncrease skill;

    @BeforeEach
    void setUp() {
        skill = new ArmourIncrease();
        skill.postConstruct();
    }

    @Test
    void name_is_plus_AV() {
        assertEquals("+AV", skill.getName());
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
        RulesCollection annotation = ArmourIncrease.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
