package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.Incorporeal;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class IncorporealSkillTest {

    private Incorporeal skill;

    @BeforeEach
    void setUp() {
        skill = new Incorporeal();
        skill.postConstruct();
    }

    @Test
    void name_is_Incorporeal() {
        assertEquals("Incorporeal", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_add_strength_to_dodge_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canAddStrengthToDodge));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = Incorporeal.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
