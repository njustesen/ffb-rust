package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.ThrowTeamMate;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ThrowTeamMateSkillTest {

    private ThrowTeamMate skill;

    @BeforeEach
    void setUp() {
        skill = new ThrowTeamMate();
        skill.postConstruct();
    }

    @Test
    void name_is_Throw_Team_Mate() {
        assertEquals("Throw Team-Mate", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_can_throw_team_mates_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canThrowTeamMates));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = ThrowTeamMate.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
