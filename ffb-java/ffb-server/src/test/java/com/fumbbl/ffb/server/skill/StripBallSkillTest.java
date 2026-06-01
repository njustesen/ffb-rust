package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.StripBall;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StripBallSkillTest {

    private StripBall skill;

    @BeforeEach
    void setUp() {
        skill = new StripBall();
        skill.postConstruct();
    }

    @Test
    void name_is_Strip_Ball() {
        assertEquals("Strip Ball", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_force_opponent_to_drop_ball_on_pushback_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.forceOpponentToDropBallOnPushback));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = StripBall.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
