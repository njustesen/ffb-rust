package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.PassBlock;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PassBlockSkillTest {

    private PassBlock skill;

    @BeforeEach
    void setUp() {
        skill = new PassBlock();
        skill.postConstruct();
    }

    @Test
    void name_is_Pass_Block() {
        assertEquals("Pass Block", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_can_move_when_opponent_passes_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveWhenOpponentPasses));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = PassBlock.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
