package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.BallAndChain;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BallAndChainSkillTest {

    private BallAndChain skill;

    @BeforeEach
    void setUp() {
        skill = new BallAndChain();
        skill.postConstruct();
    }

    @Test
    void name_is_Ball_and_Chain() {
        assertEquals("Ball and Chain", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_force_full_movement_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.forceFullMovement));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = BallAndChain.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
