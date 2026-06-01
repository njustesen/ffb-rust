package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.bb2016.MonstrousMouth;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MonstrousMouthSkillTest {

    private MonstrousMouth skill;

    @BeforeEach
    void setUp() {
        skill = new MonstrousMouth();
        skill.postConstruct();
    }

    @Test
    void name_is_Monstrous_Mouth() {
        assertEquals("Monstrous Mouth", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void skill_properties_are_not_null() {
        assertNotNull(skill.getSkillProperties());
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = MonstrousMouth.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
