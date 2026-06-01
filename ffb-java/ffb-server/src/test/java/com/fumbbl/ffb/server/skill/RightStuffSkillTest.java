package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.RightStuff;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class RightStuffSkillTest {

    private RightStuff skill;

    @BeforeEach
    void setUp() {
        skill = new RightStuff();
        skill.postConstruct();
    }

    @Test
    void name_is_Right_Stuff() {
        assertEquals("Right Stuff", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_can_be_thrown_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canBeThrown));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = RightStuff.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
