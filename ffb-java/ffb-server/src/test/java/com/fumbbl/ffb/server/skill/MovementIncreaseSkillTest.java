package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.common.MovementIncrease;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MovementIncreaseSkillTest {

    private MovementIncrease skill;

    @BeforeEach
    void setUp() {
        skill = new MovementIncrease();
        skill.postConstruct();
    }

    @Test
    void name_is_plus_MA() {
        assertEquals("+MA", skill.getName());
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
    void is_common_across_all_editions() {
        RulesCollection annotation = MovementIncrease.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
