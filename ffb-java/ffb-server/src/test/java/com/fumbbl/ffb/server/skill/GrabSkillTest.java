package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Grab;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class GrabSkillTest {

    private Grab skill;

    @BeforeEach
    void setUp() {
        skill = new Grab();
        skill.postConstruct();
    }

    @Test
    void name_is_Grab() {
        assertEquals("Grab", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_push_back_to_any_square_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPushBackToAnySquare));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Grab.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
