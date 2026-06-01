package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.SneakyGit;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SneakyGitSkillTest {

    private SneakyGit skill;

    @BeforeEach
    void setUp() {
        skill = new SneakyGit();
        skill.postConstruct();
    }

    @Test
    void name_is_Sneaky_Git() {
        assertEquals("Sneaky Git", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_always_assist_fouls_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canAlwaysAssistFouls));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = SneakyGit.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
