package com.github.fmaiassistent.steam;

import com.github.fmaiassistent.platform.OperatingSystem;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

class SteamLibrariesTest {
    @Test
    void readsLibrariesFromLibraryFolders(@TempDir Path home) throws Exception {
        Path customLib = home.resolve("mnt/bcache/Steam");
        Path steamApps = home.resolve(".local/share/Steam/steamapps");
        Files.createDirectories(steamApps);
        Files.writeString(steamApps.resolve("libraryfolders.vdf"),
                "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"" + home.resolve(".local/share/Steam") + "\"\n\t}\n"
                        + "\t\"1\"\n\t{\n\t\t\"path\"\t\t\"" + customLib + "\"\n\t}\n}");

        List<Path> libraries = SteamLibraries.libraries(home, OperatingSystem.LINUX);

        assertThat(libraries).contains(
                home.resolve(".local/share/Steam").toAbsolutePath().normalize(),
                customLib.toAbsolutePath().normalize());
    }

    @Test
    void buildsProtonUserDirectoriesPerLibrary(@TempDir Path home) {
        List<Path> userDirs = SteamLibraries.fmProtonUserDirectories(home, OperatingSystem.LINUX);

        assertThat(userDirs).allMatch(path -> path.endsWith(
                "steamapps/compatdata/4530370/pfx/drive_c/users/steamuser"));
        assertThat(userDirs.getFirst().startsWith(home.resolve(".local/share/Steam"))).isTrue();
    }

    @Test
    void returnsEmptyWhenVdfIsMissing(@TempDir Path home) {
        assertThat(SteamLibraries.parseLibraryFolders(home.resolve("libraryfolders.vdf"))).isEmpty();
    }

    @Test
    void findsLibraryHoldingTheGame(@TempDir Path home) throws Exception {
        Path customLib = home.resolve("mnt/bcache/Steam");
        Path steamApps = home.resolve(".local/share/Steam/steamapps");
        Files.createDirectories(steamApps);
        Files.createDirectories(customLib.resolve("steamapps"));
        Files.writeString(steamApps.resolve("libraryfolders.vdf"),
                "\"libraryfolders\"\n{\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"" + customLib + "\"\n\t}\n}");
        Files.writeString(customLib.resolve("steamapps/appmanifest_4530370.acf"), "\"AppState\"{}");

        assertThat(SteamLibraries.libraryWithApp(home, 4530370, OperatingSystem.LINUX))
                .hasValue(customLib.toAbsolutePath().normalize());
        assertThat(SteamLibraries.libraryWithApp(home, 12345, OperatingSystem.LINUX)).isEmpty();
    }

    @Test
    void parsesBuildIdFromAppManifest(@TempDir Path home) throws Exception {
        writeLibraryWithManifest(home,
                "\"AppState\"\n{\n\t\"appid\"\t\t\"4530370\"\n\t\"name\"\t\t\"Football Manager 27\"\n"
                        + "\t\"buildid\"\t\t\"26227640\"\n}");

        assertThat(SteamLibraries.installedBuildId(home, 4530370, OperatingSystem.LINUX)).hasValue(26227640L);
    }

    @Test
    void findsBuildInWindowsSteamInstall(@TempDir Path home) throws Exception {
        Path steam = home.resolve("Program Files (x86)/Steam");
        Path steamApps = steam.resolve("steamapps");
        Files.createDirectories(steamApps);
        Files.writeString(steamApps.resolve("libraryfolders.vdf"),
                "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\""
                        + steam.toString().replace("\\", "\\\\") + "\"\n\t}\n}");
        Files.writeString(steamApps.resolve("appmanifest_4530370.acf"),
                "\"AppState\"\n{\n\t\"buildid\"\t\t\"26227640\"\n}");

        assertThat(SteamLibraries.libraries(home, List.of(steam))).contains(steam.toAbsolutePath().normalize());
        assertThat(SteamLibraries.parseLibraryFolders(steamApps.resolve("libraryfolders.vdf")))
                .contains(steam.toAbsolutePath().normalize());
        String previous = System.getProperty(SteamLibraries.DIRECTORY_PROPERTY);
        System.setProperty(SteamLibraries.DIRECTORY_PROPERTY, steam.toString());
        try {
            assertThat(SteamLibraries.installedBuildId(home, 4530370)).hasValue(26227640L);
        } finally {
            if (previous == null) {
                System.clearProperty(SteamLibraries.DIRECTORY_PROPERTY);
            } else {
                System.setProperty(SteamLibraries.DIRECTORY_PROPERTY, previous);
            }
        }
    }

    @Test
    void emptyBuildIdWhenManifestMissing(@TempDir Path home) {
        assertThat(SteamLibraries.installedBuildId(home, 12345)).isEmpty();
    }

    @Test
    void doesNotUseTheStillInstalledPreviousGameManifest(@TempDir Path home) throws Exception {
        Path steamApps = home.resolve(".local/share/Steam/steamapps");
        Files.createDirectories(steamApps);
        Files.writeString(steamApps.resolve("appmanifest_3551340.acf"),
                "\"AppState\"\n{\n\t\"buildid\"\t\t\"23583635\"\n}");

        assertThat(SteamLibraries.installedBuildId(home, SteamLibraries.FM27_APP_ID)).isEmpty();
        assertThat(SteamLibraries.libraryWithApp(home, SteamLibraries.FM27_APP_ID)).isEmpty();
    }

    @Test
    void emptyBuildIdWhenManifestLacksBuildId(@TempDir Path home) throws Exception {
        writeLibraryWithManifest(home, "\"AppState\"{}");

        assertThat(SteamLibraries.installedBuildId(home, 4530370, OperatingSystem.LINUX)).isEmpty();
    }

    @Test
    void emptyBuildIdWhenManifestOversized(@TempDir Path home) throws Exception {
        writeLibraryWithManifest(home, "\"AppState\"\n{\n\t\"buildid\"\t\t\"1\"\n}\n" + "x".repeat(64 * 1024));

        assertThat(SteamLibraries.installedBuildId(home, 4530370, OperatingSystem.LINUX)).isEmpty();
    }

    @Test
    void macReadsCustomLibrariesFromNativeSteamRoot(@TempDir Path home) throws Exception {
        Path steam = home.resolve("Library/Application Support/Steam");
        Path custom = home.resolve("External Volume/SteamLibrary");
        Files.createDirectories(steam.resolve("steamapps"));
        Files.createDirectories(custom.resolve("steamapps"));
        Files.writeString(steam.resolve("steamapps/libraryfolders.vdf"),
                "\"libraryfolders\" { \"0\" { \"path\" \"" + custom + "\" } }");
        Files.writeString(custom.resolve("steamapps/appmanifest_4530370.acf"),
                "\"AppState\" { \"buildid\" \"26227640\" }");

        assertThat(SteamLibraries.libraries(home, OperatingSystem.MACOS))
                .containsExactly(custom.toAbsolutePath().normalize(), steam.toAbsolutePath().normalize());
        assertThat(SteamLibraries.installedBuildId(home, 4530370, OperatingSystem.MACOS)).hasValue(26227640L);
        assertThat(SteamLibraries.fmProtonUserDirectories(home, OperatingSystem.MACOS)).isEmpty();
    }

    private static Path writeLibraryWithManifest(Path home, String manifest) throws Exception {
        Path customLib = home.resolve("mnt/bcache/Steam");
        Path steamApps = home.resolve(".local/share/Steam/steamapps");
        Files.createDirectories(steamApps);
        Files.createDirectories(customLib.resolve("steamapps"));
        Files.writeString(steamApps.resolve("libraryfolders.vdf"),
                "\"libraryfolders\"\n{\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"" + customLib + "\"\n\t}\n}");
        Files.writeString(customLib.resolve("steamapps/appmanifest_4530370.acf"), manifest);
        return customLib;
    }
}
