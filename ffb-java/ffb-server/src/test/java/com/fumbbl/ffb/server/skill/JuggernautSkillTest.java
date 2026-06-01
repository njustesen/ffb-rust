package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Juggernaut;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class JuggernautSkillTest {

    private Juggernaut skill;

    @BeforeEach
    void setUp() {
        skill = new Juggernaut();
        skill.postConstruct();
    }

    @Test
    void name_is_Juggernaut() {
        assertEquals("Juggernaut", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_convert_both_down_to_push_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canConvertBothDownToPush));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Juggernaut.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
