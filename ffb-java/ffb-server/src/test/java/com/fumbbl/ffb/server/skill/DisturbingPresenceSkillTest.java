package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.DisturbingPresence;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DisturbingPresenceSkillTest {

    private DisturbingPresence skill;

    @BeforeEach
    void setUp() {
        skill = new DisturbingPresence();
        skill.postConstruct();
    }

    @Test
    void name_is_Disturbing_Presence() {
        assertEquals("Disturbing Presence", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_inflicts_disturbing_presence_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.inflictsDisturbingPresence));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = DisturbingPresence.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
