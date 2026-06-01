package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.ReallyStupid;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ReallyStupidSkillTest {

    private ReallyStupid skill;

    @BeforeEach
    void setUp() {
        skill = new ReallyStupid();
        skill.postConstruct();
    }

    @Test
    void name_is_Really_Stupid() {
        assertEquals("Really Stupid", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_applies_confusion_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.appliesConfusion));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = ReallyStupid.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
