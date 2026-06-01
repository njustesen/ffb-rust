package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.DivingTackle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DivingTackleSkillTest {

    private DivingTackle skill;

    @BeforeEach
    void setUp() {
        skill = new DivingTackle();
        skill.postConstruct();
    }

    @Test
    void name_is_Diving_Tackle() {
        assertEquals("Diving Tackle", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_attempt_to_tackle_dodging_player_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canAttemptToTackleDodgingPlayer));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = DivingTackle.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
