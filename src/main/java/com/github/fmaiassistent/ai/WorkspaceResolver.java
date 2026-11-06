package com.github.fmaiassistent.ai;

import com.github.fmaiassistent.FmAiAssistentApplication;
import com.github.fmaiassistent.platform.OperatingSystem;
import org.springframework.boot.system.ApplicationHome;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/** Resolves provider workspaces consistently for IDE runs and packaged applications. */
public final class WorkspaceResolver {
    private static final int MAX_PARENT_DEPTH = 8;

    private WorkspaceResolver() {
    }

    public static Path resolve(String configuredDirectory) {
        String value = configuredDirectory == null || configuredDirectory.isBlank() ? "." : configuredDirectory;
        Path configured = Path.of(value);
        return configured.isAbsolute()
                ? configured.normalize()
                : applicationDirectory().resolve(configured).normalize();
    }

    static Path findProjectRoot(Path start) {
        Path candidate = start;
        for (int depth = 0; candidate != null && depth < MAX_PARENT_DEPTH; depth++, candidate = candidate.getParent()) {
            if (candidate.getFileName() != null && candidate.getFileName().toString().endsWith(".app")) {
                return null;
            }
            if (Files.isRegularFile(candidate.resolve("pom.xml")) || Files.isDirectory(candidate.resolve(".git"))) {
                return candidate;
            }
        }
        return null;
    }

    private static Path applicationDirectory() {
        try {
            Path home = new ApplicationHome(FmAiAssistentApplication.class)
                    .getDir()
                    .toPath()
                    .toAbsolutePath()
                    .normalize();
            Path root = findProjectRoot(home);
            if (root != null) {
                return root;
            }
            Path workspace = packagedDirectory(home, Path.of(System.getProperty("user.home")),
                    OperatingSystem.current());
            Files.createDirectories(workspace);
            return workspace;
        } catch (IOException exception) {
            throw new IllegalStateException("Could not create the assistant workspace", exception);
        } catch (RuntimeException ignored) {
            return Path.of(System.getProperty("user.dir")).toAbsolutePath().normalize();
        }
    }

    static Path packagedDirectory(Path applicationHome, Path userHome, OperatingSystem os) {
        return os == OperatingSystem.MACOS
                ? userHome.resolve("Library/Application Support/FM27 Assistant/workspace")
                        .toAbsolutePath().normalize()
                : applicationHome;
    }
}
