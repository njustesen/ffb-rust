package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.special.TeamCaptain;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TeamCaptainSkillTest {

    private TeamCaptain skill;

    @BeforeEach
    void setUp() {
        skill = new TeamCaptain();
        skill.postConstruct();
    }

    @Test
    void name_is_Team_Captain() {
        assertEquals("Team Captain", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_save_re_rolls_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canSaveReRolls));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = TeamCaptain.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
