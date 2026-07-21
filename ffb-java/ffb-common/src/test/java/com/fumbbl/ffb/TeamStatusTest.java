package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/team_status.rs for {@link TeamStatus}.
 */
public class TeamStatusTest {

	private static final TeamStatus[] STATUSES = {
		TeamStatus.NEW, TeamStatus.ACTIVE, TeamStatus.PENDING_APPROVAL,
		TeamStatus.BLOCKED, TeamStatus.RETIRED, TeamStatus.WAITING_FOR_OPPONENT,
		TeamStatus.SKILL_ROLLS_PENDING,
	};

	@Test
	public void allIdsAreUnique() {
		List<Integer> ids = Arrays.stream(STATUSES).map(TeamStatus::getId).collect(Collectors.toList());
		Set<Integer> unique = new HashSet<>(ids);
		assertEquals(ids.size(), unique.size());
	}

	@Test
	public void idsAreSequentialFromZero() {
		List<Integer> ids = Arrays.stream(STATUSES).map(TeamStatus::getId).sorted().collect(Collectors.toList());
		assertEquals(Arrays.asList(0, 1, 2, 3, 4, 5, 6), ids);
	}

	@Test
	public void allNamesAreNonEmptyAndDistinct() {
		List<String> names = Arrays.stream(STATUSES).map(TeamStatus::getName).collect(Collectors.toList());
		for (String name : names) {
			assertFalse(name.isEmpty());
		}
		Set<String> unique = new HashSet<>(names);
		assertEquals(names.size(), unique.size());
	}
}
