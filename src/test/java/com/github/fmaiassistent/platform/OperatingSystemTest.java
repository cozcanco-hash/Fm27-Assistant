package com.github.fmaiassistent.platform;

import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.assertj.core.api.Assertions.assertThat;

class OperatingSystemTest {
    @ParameterizedTest
    @CsvSource({"Mac OS X,MACOS", "Darwin,MACOS", "macOS,MACOS", "Windows 11,WINDOWS", "Linux,LINUX", "FreeBSD,OTHER"})
    void recognizesPlatformsWithoutConfusingDarwinWithWindows(String name, OperatingSystem expected) {
        assertThat(OperatingSystem.fromName(name)).isEqualTo(expected);
    }
}
