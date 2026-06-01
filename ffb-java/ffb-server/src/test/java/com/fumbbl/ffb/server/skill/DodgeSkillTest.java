package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Dodge;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.*;

class DodgeSkillTest {

    private Dodge skill;

    @BeforeEach
    void setUp() {
        skill = new Dodge();
        skill.postConstruct();
    }

    @Test
    void name_is_Dodge() {
        assertEquals("Dodge", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_ignoreDefenderStumblesResult_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.ignoreDefenderStumblesResult),
            "Dodge must register ignoreDefenderStumblesResult so the Defender Stumbles result is treated as a push");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Dodge does not force follow-up");
    }

    @Test
    void is_bb2016_and_bb2020_edition() {
        RulesCollection[] annotations = Dodge.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertTrue(Arrays.stream(annotations).anyMatch(a -> a.value() == RulesCollection.Rules.BB2016),
            "Dodge must be available in BB2016");
        assertTrue(Arrays.stream(annotations).anyMatch(a -> a.value() == RulesCollection.Rules.BB2020),
            "Dodge must be available in BB2020");
    }
}
