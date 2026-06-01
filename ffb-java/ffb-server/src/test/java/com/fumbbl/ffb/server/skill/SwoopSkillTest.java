package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Swoop;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SwoopSkillTest {

    private Swoop skill;

    @BeforeEach
    void setUp() {
        skill = new Swoop();
        skill.postConstruct();
    }

    @Test
    void name_is_Swoop() {
        assertEquals("Swoop", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_prevent_stunty_dodge_modifier_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.preventStuntyDodgeModifier));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Swoop.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
