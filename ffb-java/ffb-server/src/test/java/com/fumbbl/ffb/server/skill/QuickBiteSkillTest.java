package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.special.QuickBite;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class QuickBiteSkillTest {

    private QuickBite skill;

    @BeforeEach
    void setUp() {
        skill = new QuickBite();
        skill.postConstruct();
    }

    @Test
    void name_is_QuickBite() {
        assertEquals("Quick Bite", skill.getName());
    }

    @Test
    void category_is_TRAIT() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_two_edition_annotations() {
        RulesCollection[] annotations = QuickBite.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertEquals(2, annotations.length);
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
