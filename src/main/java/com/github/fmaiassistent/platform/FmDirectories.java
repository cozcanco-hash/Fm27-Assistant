package com.github.fmaiassistent.platform;

import java.nio.file.Path;

/** Native user data locations, independent of where Steam installs the game. */
public final class FmDirectories {
    private FmDirectories() { }

    public static Path nativeUserData(Path home, OperatingSystem os) {
        String parent = os == OperatingSystem.MACOS ? "Library/Application Support" : "Documents";
        return home.resolve(parent).resolve("Sports Interactive/Football Manager 27")
                .toAbsolutePath().normalize();
    }
}
