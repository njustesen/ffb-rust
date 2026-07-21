package com.fumbbl.ffb.client.state.logic.interaction;

import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.Influences;
import com.fumbbl.ffb.model.skill.Skill;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Arrays;
import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(MockitoExtension.class)
class ActionContextTest {

	@Mock
	private Skill blockSkill;

	@Mock
	private Skill otherSkill;

	@Test
	public void testNewContextHasEmptyLists() {
		ActionContext ctx = new ActionContext();
		assertTrue(ctx.getActions().isEmpty());
		assertTrue(ctx.getInfluences().isEmpty());
		assertTrue(ctx.getBlockAlternatives().isEmpty());
	}

	@Test
	public void testAddActionAndInfluenceAndSkill() {
		ActionContext ctx = new ActionContext();
		ctx.add(ClientAction.MOVE);
		ctx.add(Influences.HAS_ACTED);
		ctx.add(blockSkill);

		assertEquals(Collections.singletonList(ClientAction.MOVE), ctx.getActions());
		assertEquals(Collections.singletonList(Influences.HAS_ACTED), ctx.getInfluences());
		assertEquals(1, ctx.getBlockAlternatives().size());
	}

	@Test
	public void testMergeCombinesListsFromBothContexts() {
		ActionContext ctx = new ActionContext();
		ctx.add(ClientAction.MOVE);
		ctx.add(Influences.HAS_ACTED);

		ActionContext other = new ActionContext();
		other.add(ClientAction.BLOCK);
		other.add(Influences.IS_JUMPING);
		other.add(otherSkill);

		ctx.merge(other);

		assertEquals(Arrays.asList(ClientAction.MOVE, ClientAction.BLOCK), ctx.getActions());
		assertEquals(Arrays.asList(Influences.HAS_ACTED, Influences.IS_JUMPING), ctx.getInfluences());
		assertEquals(1, ctx.getBlockAlternatives().size());
	}

	@Test
	public void testMergeReturnsMutableReferenceToSelf() {
		ActionContext ctx = new ActionContext();
		ActionContext other = new ActionContext();
		ActionContext merged = ctx.merge(other);
		assertSame(ctx, merged);
		assertTrue(merged.getActions().isEmpty());
	}
}
