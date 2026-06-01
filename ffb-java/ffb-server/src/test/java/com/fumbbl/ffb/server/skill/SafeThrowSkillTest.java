package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.SafeThrow;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SafeThrowSkillTest {

    private SafeThrow skill;

    @BeforeEach
    void setUp() {
        skill = new SafeThrow();
        skill.postConstruct();
    }

    @Test
    void name_is_Safe_Throw() {
        assertEquals("Safe Throw", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_cancel_interceptions_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canCancelInterceptions));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = SafeThrow.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
