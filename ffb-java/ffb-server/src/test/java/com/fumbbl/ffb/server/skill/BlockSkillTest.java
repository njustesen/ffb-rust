package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Block;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BlockSkillTest {

    private Block skill;

    @BeforeEach
    void setUp() {
        skill = new Block();
        skill.postConstruct();
    }

    @Test
    void name_is_Block() {
        assertEquals("Block", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_preventFallOnBothDown_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.preventFallOnBothDown),
            "Block must register preventFallOnBothDown so Both Down result does not knock attacker down");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Block does not force follow-up (that is Frenzy)");
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Block.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Block must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
