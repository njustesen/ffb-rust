package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Tentacles;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TentaclesSkillTest {

    private Tentacles skill;

    @BeforeEach
    void setUp() {
        skill = new Tentacles();
        skill.postConstruct();
    }

    @Test
    void name_is_Tentacles() {
        assertEquals("Tentacles", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_can_hold_players_leaving_tacklezones_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Tentacles.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
