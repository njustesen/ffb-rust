package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Wrestle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WrestleSkillTest {

    private Wrestle skill;

    @BeforeEach
    void setUp() {
        skill = new Wrestle();
        skill.postConstruct();
    }

    @Test
    void name_is_Wrestle() {
        assertEquals("Wrestle", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_canTakeDownPlayersWithHimOnBothDown_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canTakeDownPlayersWithHimOnBothDown),
            "Wrestle must register canTakeDownPlayersWithHimOnBothDown to enable both-prone on Both Down result");
    }

    @Test
    void does_not_have_preventFallOnBothDown_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.preventFallOnBothDown),
            "Wrestle does not prevent attacker from falling (that is Block)");
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Wrestle.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Wrestle must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
