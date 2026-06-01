package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Guard;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class GuardSkillTest {

    private Guard skill;

    @BeforeEach
    void setUp() {
        skill = new Guard();
        skill.postConstruct();
    }

    @Test
    void name_is_Guard() {
        assertEquals("Guard", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_assistsBlocksInTacklezones_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.assistsBlocksInTacklezones),
            "Guard must register assistsBlocksInTacklezones so player assists even while in an opponent's tackle zone");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Guard does not force follow-up");
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Guard.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Guard must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
