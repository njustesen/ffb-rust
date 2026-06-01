package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2020.special.WhirlingDervish;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WhirlingDervishSkillTest {

    private WhirlingDervish skill;

    @BeforeEach
    void setUp() {
        skill = new WhirlingDervish();
        skill.postConstruct();
    }

    @Test
    void name_is_Whirling_Dervish() {
        assertEquals("Whirling Dervish", skill.getName());
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
        RulesCollection annotation = WhirlingDervish.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
