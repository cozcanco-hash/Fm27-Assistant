package com.github.fmaiassistent.ai;

import com.github.fmaiassistent.platform.OperatingSystem;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.assertj.core.api.Assertions.assertThat;

class WorkspaceResolverTest {
    @TempDir
    Path temporaryDirectory;

    @Test
    void findsNearestProjectMarkerWithinBoundedParentWalk() throws Exception {
        Path root = Files.createDirectories(temporaryDirectory.resolve("project"));
        Files.createFile(root.resolve("pom.xml"));
        Path nested = Files.createDirectories(root.resolve("a/b/c"));

        assertThat(WorkspaceResolver.findProjectRoot(nested)).isEqualTo(root);
    }

    @Test
    void normalizesAbsoluteConfiguredDirectory() {
        Path configured = temporaryDirectory.resolve("first/../second");

        assertThat(WorkspaceResolver.resolve(configured.toString())).isEqualTo(configured.normalize());
    }

    @Test
    void macAppUsesWritableUserWorkspaceOutsideTheBundle() {
        Path bundle = temporaryDirectory.resolve("Applications/FM AI Assistent.app/Contents/app");
        Path home = temporaryDirectory.resolve("user");

        assertThat(WorkspaceResolver.packagedDirectory(bundle, home, OperatingSystem.MACOS))
                .isEqualTo(home.resolve("Library/Application Support/FM27 Assistant/workspace"));
        assertThat(WorkspaceResolver.packagedDirectory(bundle, home, OperatingSystem.LINUX))
                .isEqualTo(bundle);
    }

    @Test
    void packagedMacAppDoesNotMistakeItsBuildRepositoryForAWorkspace() throws Exception {
        Files.createFile(temporaryDirectory.resolve("pom.xml"));
        Path bundle = Files.createDirectories(
                temporaryDirectory.resolve("target/dist/FM AI Assistent.app/Contents/app"));

        assertThat(WorkspaceResolver.findProjectRoot(bundle)).isNull();
    }
}
