package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.ViolentInnovator;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ViolentInnovatorSkillTest {

    private ViolentInnovator skill;

    @BeforeEach
    void setUp() {
        skill = new ViolentInnovator();
        skill.postConstruct();
    }

    @Test
    void name_is_Violent_Innovator() {
        assertEquals("Violent Innovator", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_grants_spp_from_special_actions_cas_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.grantsSppFromSpecialActionsCas));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = ViolentInnovator.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
