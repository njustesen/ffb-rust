package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.PutTheBootIn;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PutTheBootInSkillTest {

    private PutTheBootIn skill;

    @BeforeEach
    void setUp() {
        skill = new PutTheBootIn();
        skill.postConstruct();
    }

    @Test
    void name_is_Put_the_Boot_In() {
        assertEquals("Put the Boot In", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_can_always_assist_fouls_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canAlwaysAssistFouls));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = PutTheBootIn.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
