package com.github.fmaiassistent.codex;

import com.github.fmaiassistent.platform.OperatingSystem;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.assertj.core.api.Assertions.assertThat;

class CodexExecutableResolverTest {
    @TempDir
    Path home;

    @Test
    void explicitExecutableWinsOverDiscoveredCopies() throws Exception {
        Path fallback = executable(home.resolve(".local/bin/codex"));
        String configured = home.resolve("custom/codex").toString();

        assertThat(CodexExecutableResolver.resolve(configured, home,
                fallback.getParent().toString(), OperatingSystem.MACOS)).isEqualTo(configured);
    }

    @Test
    void pathExecutableWinsOverHomeFallback() throws Exception {
        executable(home.resolve(".local/bin/codex"));
        Path onPath = executable(home.resolve("preferred/codex"));

        assertThat(CodexExecutableResolver.resolve("codex", home,
                onPath.getParent().toString(), OperatingSystem.MACOS)).isEqualTo(onPath.toString());
    }

    @Test
    void finderWithMinimalPathCanFindUserInstallation() throws Exception {
        Path expected = executable(home.resolve(".local/bin/codex"));

        assertThat(CodexExecutableResolver.resolve("codex", home, "", OperatingSystem.MACOS))
                .isEqualTo(expected.toString());
    }

    @Test
    void executableDirectoryIsNotMistakenForTheCodexBinary() throws Exception {
        Path invalid = Files.createDirectories(home.resolve("bad/codex"));
        Path expected = executable(home.resolve("good/codex"));

        assertThat(CodexExecutableResolver.resolve("codex", home,
                invalid.getParent() + File.pathSeparator + expected.getParent(), OperatingSystem.MACOS))
                .isEqualTo(expected.toString());
    }

    @Test
    void macFallbacksIncludeAppleSiliconAndIntelHomebrew() {
        assertThat(CodexExecutableResolver.fallbackCandidates(home, OperatingSystem.MACOS))
                .contains(Path.of("/opt/homebrew/bin/codex"), Path.of("/usr/local/bin/codex"));
        assertThat(CodexExecutableResolver.fallbackCandidates(home, OperatingSystem.LINUX))
                .doesNotContain(Path.of("/opt/homebrew/bin/codex"));
    }

    @Test
    void customCommandIsNotSilentlyReplacedByCodex() throws Exception {
        executable(home.resolve(".local/bin/codex"));

        assertThat(CodexExecutableResolver.resolve("custom-agent", home, "", OperatingSystem.MACOS))
                .isEqualTo("custom-agent");
    }

    private static Path executable(Path path) throws Exception {
        Files.createDirectories(path.getParent());
        Files.writeString(path, "#!/bin/sh\nexit 0\n");
        assertThat(path.toFile().setExecutable(true)).isTrue();
        return path;
    }
}
