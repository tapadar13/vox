import { describe, expect, it, vi } from "vitest";

import { formatBytes, formatDuration, formatHotkey, relativeDate } from "./format";

describe("formatDuration", () => {
  it("uses compact units from milliseconds through hours", () => {
    expect(formatDuration(245)).toBe("245ms");
    expect(formatDuration(12_400)).toBe("12s");
    expect(formatDuration(125_000)).toBe("2m");
    expect(formatDuration(7_500_000)).toBe("2h 5m");
  });
});

describe("formatBytes", () => {
  it("keeps model sizes readable", () => {
    expect(formatBytes(512_000)).toBe("512 KB");
    expect(formatBytes(59_700_000)).toBe("59.7 MB");
    expect(formatBytes(574_000_000)).toBe("574 MB");
  });
});

describe("formatHotkey", () => {
  it("renders macOS modifier glyphs", () => {
    expect(formatHotkey("CommandOrControl+Shift+Space")).toBe("⌘ ⇧ Space");
    expect(formatHotkey("Control+Alt+K")).toBe("⌃ ⌥ K");
  });
});

describe("relativeDate", () => {
  it("formats recent local history without a network clock", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-08-26T10:00:00.000Z"));
    expect(relativeDate("2026-08-26T09:59:45.000Z")).toBe("Just now");
    expect(relativeDate("2026-08-26T09:42:00.000Z")).toBe("18m ago");
    expect(relativeDate("2026-08-25T10:00:00.000Z")).toBe("1d ago");
    vi.useRealTimers();
  });
});
