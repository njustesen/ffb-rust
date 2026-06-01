package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Swarming;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SwarmingSkillTest {

    private Swarming skill;

    @BeforeEach
    void setUp() {
        skill = new Swarming();
        skill.postConstruct();
    }

    @Test
    void name_is_Swarming() {
        assertEquals("Swarming", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_can_sneak_extra_players_onto_pitch_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canSneakExtraPlayersOntoPitch));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Swarming.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
