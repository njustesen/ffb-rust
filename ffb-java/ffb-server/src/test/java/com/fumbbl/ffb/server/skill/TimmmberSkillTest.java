package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Timmmber;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TimmmberSkillTest {

    private Timmmber skill;

    @BeforeEach
    void setUp() {
        skill = new Timmmber();
        skill.postConstruct();
    }

    @Test
    void name_is_Timmm_ber() {
        assertEquals("Timmm-ber!", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_allow_stand_up_assists_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.allowStandUpAssists));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Timmmber.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
