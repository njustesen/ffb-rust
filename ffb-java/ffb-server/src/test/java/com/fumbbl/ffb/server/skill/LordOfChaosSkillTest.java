package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.LordOfChaos;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class LordOfChaosSkillTest {

    private LordOfChaos skill;

    @BeforeEach
    void setUp() {
        skill = new LordOfChaos();
        skill.postConstruct();
    }

    @Test
    void name_is_Lord_of_Chaos() {
        assertEquals("Lord of Chaos", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_grants_single_use_team_reroll_when_on_pitch_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.grantsSingleUseTeamRerollWhenOnPitch));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = LordOfChaos.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
