package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/file_iterator.rs for {@link FileIterator}.
 */
public class FileIteratorTest {

	@Test
	void iteratesFilesInTempDir() throws IOException {
		Path dir = Files.createTempDirectory("ffb_file_iter_test");
		Path filePath = dir.resolve("test.txt");
		Files.write(filePath, "test".getBytes());

		FileIterator iter = new FileIterator(dir.toFile());
		List<File> found = new ArrayList<>();
		while (iter.hasNext()) {
			found.add(iter.next());
		}
		assertTrue(found.stream().anyMatch(p -> "test.txt".equals(p.getName())));

		Files.deleteIfExists(filePath);
		Files.deleteIfExists(dir);
	}

	@Test
	void knownSizeCountsFiles() throws IOException {
		Path dir = Files.createTempDirectory("ffb_file_iter_size");
		Path f1 = dir.resolve("a.txt");
		Path f2 = dir.resolve("b.txt");
		Files.write(f1, "a".getBytes());
		Files.write(f2, "b".getBytes());

		FileIterator iter = new FileIterator(dir.toFile());
		while (iter.hasNext()) {
			iter.next();
		}
		assertTrue(iter.knownSize() >= 2);

		Files.deleteIfExists(f1);
		Files.deleteIfExists(f2);
		Files.deleteIfExists(dir);
	}

}
