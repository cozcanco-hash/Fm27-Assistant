package com.github.fmaiassistent.codex;

import com.github.fmaiassistent.platform.OperatingSystem;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

@Component
class CodexExecutableResolver {
    private static final Logger log = LoggerFactory.getLogger(CodexExecutableResolver.class);

    private final CodexProperties properties;

    CodexExecutableResolver(CodexProperties properties) {
        this.properties = properties;
    }

    String resolve() {
        return resolve(properties.executable(), Path.of(System.getProperty("user.home")),
                System.getenv().getOrDefault("PATH", ""), OperatingSystem.current());
    }

    static String resolve(String configured, Path home, String searchPath, OperatingSystem os) {
        if (configured.contains(File.separator) || Path.of(configured).isAbsolute()) {
            return configured;
        }

        for (String directory : searchPath.split(File.pathSeparator)) {
            if (!directory.isBlank()) {
                Path candidate = Path.of(directory, configured);
                if (isExecutableFile(candidate)) {
                    return candidate.toString();
                }
            }
        }

        if (!"codex".equals(configured)) {
            return configured;
        }
        List<Path> candidates = new ArrayList<>(fallbackCandidates(home, os));
        Path nvmVersions = home.resolve(".nvm/versions/node");
        if (Files.isDirectory(nvmVersions)) {
            try (var versions = Files.list(nvmVersions)) {
                versions.map(version -> version.resolve("bin/codex"))
                        .filter(CodexExecutableResolver::isExecutableFile)
                        .sorted(Comparator.comparingLong(CodexExecutableResolver::modified).reversed())
                        .forEach(candidates::add);
            } catch (IOException ex) {
                log.debug("Could not inspect nvm installations for Codex", ex);
            }
        }
        return candidates.stream()
                .filter(CodexExecutableResolver::isExecutableFile)
                .findFirst()
                .map(Path::toString)
                .orElse(configured);
    }

    static List<Path> fallbackCandidates(Path home, OperatingSystem os) {
        List<Path> candidates = new ArrayList<>(List.of(
                home.resolve(".local/bin/codex"),
                home.resolve(".npm-global/bin/codex"),
                home.resolve(".volta/bin/codex"),
                home.resolve(".nvm/current/bin/codex")));
        if (os == OperatingSystem.MACOS) {
            candidates.add(Path.of("/opt/homebrew/bin/codex"));
            candidates.add(Path.of("/usr/local/bin/codex"));
        }
        return List.copyOf(candidates);
    }

    private static boolean isExecutableFile(Path path) {
        return Files.isRegularFile(path) && Files.isExecutable(path);
    }

    private static long modified(Path path) {
        try {
            return Files.getLastModifiedTime(path).toMillis();
        } catch (IOException ignored) {
            return 0L;
        }
    }
}
