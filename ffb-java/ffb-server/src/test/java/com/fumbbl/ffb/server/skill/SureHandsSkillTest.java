package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.CancelSkillProperty;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.SureHands;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SureHandsSkillTest {

    private SureHands skill;

    @BeforeEach
    void setUp() {
        skill = new SureHands();
        skill.postConstruct();
    }

    @Test
    void name_is_Sure_Hands() {
        assertEquals("Sure Hands", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_cancel_stripBall_property() {
        assertTrue(skill.hasSkillProperty(new CancelSkillProperty(NamedProperties.forceOpponentToDropBallOnPushback)),
            "Sure Hands must cancel Strip Ball's forceOpponentToDropBallOnPushback");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Sure Hands does not force follow-up");
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = SureHands.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Sure Hands must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
