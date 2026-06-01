package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.KickOffReturn;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class KickOffReturnSkillTest {

    private KickOffReturn skill;

    @BeforeEach
    void setUp() {
        skill = new KickOffReturn();
        skill.postConstruct();
    }

    @Test
    void name_is_Kick_Off_Return() {
        assertEquals("Kick-Off Return", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_can_move_during_kick_off_scatter_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveDuringKickOffScatter));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = KickOffReturn.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
