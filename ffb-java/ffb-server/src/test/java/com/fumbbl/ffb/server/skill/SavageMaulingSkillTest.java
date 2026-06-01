package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.special.SavageMauling;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SavageMaulingSkillTest {

    private SavageMauling skill;

    @BeforeEach
    void setUp() {
        skill = new SavageMauling();
        skill.postConstruct();
    }

    @Test
    void name_is_SavageMauling() {
        assertEquals("Savage Mauling", skill.getName());
    }

    @Test
    void category_is_TRAIT() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_two_edition_annotations() {
        RulesCollection[] annotations = SavageMauling.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertEquals(2, annotations.length);
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
