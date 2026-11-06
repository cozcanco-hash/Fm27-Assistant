package com.github.fmaiassistent.memory;

import com.github.fmaiassistent.linux.LinuxProcessReader;
import com.github.fmaiassistent.linux.ProcessInfo;
import com.github.fmaiassistent.windows.WindowsProcessReader;
import com.github.fmaiassistent.platform.OperatingSystem;

import java.io.IOException;
import java.util.List;
import java.util.Locale;

public final class ProcessReaders {
    private ProcessReaders() {
    }

    public static ProcessMemoryReader open(int pid) throws IOException {
        requireSupportedLiveReader(OperatingSystem.current());
        if (isWindows()) {
            return new WindowsProcessReader(pid);
        }
        if (isLinux()) {
            return new LinuxProcessReader(pid);
        }
        throw new UnsupportedOperationException("Unsupported operating system: " + System.getProperty("os.name"));
    }

    public static List<ProcessInfo> findProcesses(String query) throws IOException {
        requireSupportedLiveReader(OperatingSystem.current());
        if (isWindows()) {
            return WindowsProcessReader.findProcesses(query);
        }
        if (isLinux()) {
            return LinuxProcessReader.findProcesses(query);
        }
        throw new UnsupportedOperationException("Unsupported operating system: " + System.getProperty("os.name"));
    }

    public static boolean supportsLiveData() {
        OperatingSystem os = OperatingSystem.current();
        return os == OperatingSystem.WINDOWS || os == OperatingSystem.LINUX;
    }

    public static String macOsPreviewMessage() {
        return "macOS preview: live squad loading is not available yet. "
                + "You can load tactic files and use AI chat. "
                + "A verified macOS game-data reader is required for squad advice.";
    }

    static void requireSupportedLiveReader(OperatingSystem os) throws IOException {
        if (os == OperatingSystem.MACOS) {
            throw new IOException(macOsPreviewMessage());
        }
    }

    private static boolean isWindows() {
        return System.getProperty("os.name", "").toLowerCase(Locale.ROOT).startsWith("windows");
    }

    private static boolean isLinux() {
        return System.getProperty("os.name", "").toLowerCase(Locale.ROOT).startsWith("linux");
    }
}
