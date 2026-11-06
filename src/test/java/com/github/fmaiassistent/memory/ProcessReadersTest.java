package com.github.fmaiassistent.memory;

import com.github.fmaiassistent.platform.OperatingSystem;
import org.junit.jupiter.api.Test;

import java.io.IOException;

import static org.assertj.core.api.Assertions.assertThatCode;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class ProcessReadersTest {
    @Test
    void macPreviewRejectsLiveReadsBeforeOpeningNativeMemory() {
        assertThatThrownBy(() -> ProcessReaders.requireSupportedLiveReader(OperatingSystem.MACOS))
                .isInstanceOf(IOException.class)
                .hasMessageContaining("macOS preview")
                .hasMessageContaining("live squad loading is not available");
    }

    @Test
    void existingSupportedPlatformsRemainAvailable() {
        assertThatCode(() -> ProcessReaders.requireSupportedLiveReader(OperatingSystem.WINDOWS))
                .doesNotThrowAnyException();
        assertThatCode(() -> ProcessReaders.requireSupportedLiveReader(OperatingSystem.LINUX))
                .doesNotThrowAnyException();
    }
}
