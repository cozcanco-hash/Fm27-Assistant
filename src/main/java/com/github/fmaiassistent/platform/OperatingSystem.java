package com.github.fmaiassistent.platform;

import java.util.Locale;

/** Host operating system; Darwin must never be mistaken for Windows. */
public enum OperatingSystem {
    MACOS, WINDOWS, LINUX, OTHER;

    public static OperatingSystem current() {
        return fromName(System.getProperty("os.name", ""));
    }

    public static OperatingSystem fromName(String name) {
        String value = name == null ? "" : name.strip().toLowerCase(Locale.ROOT);
        if (value.startsWith("mac") || value.equals("darwin")) return MACOS;
        if (value.startsWith("windows")) return WINDOWS;
        if (value.startsWith("linux")) return LINUX;
        return OTHER;
    }
}
