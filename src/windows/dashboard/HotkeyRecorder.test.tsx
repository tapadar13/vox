import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { HotkeyRecorder } from "./HotkeyRecorder";

describe("HotkeyRecorder", () => {
  it("captures a modified keyboard shortcut", () => {
    const onChange = vi.fn();
    render(<HotkeyRecorder value="Command+Shift+Space" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button"));
    fireEvent.keyDown(window, {
      key: "k",
      code: "KeyK",
      metaKey: true,
      shiftKey: true,
    });
    expect(onChange).toHaveBeenCalledWith("Command+Shift+K");
  });

  it("allows Escape to leave capture without replacing the shortcut", () => {
    const onChange = vi.fn();
    render(<HotkeyRecorder value="Command+Shift+Space" onChange={onChange} />);
    fireEvent.click(screen.getByRole("button"));
    expect(screen.getByRole("button")).toHaveTextContent("Press shortcut…");
    fireEvent.keyDown(window, { key: "Escape", code: "Escape" });
    expect(onChange).not.toHaveBeenCalled();
    expect(screen.getByRole("button")).toHaveTextContent("⌘ ⇧ Space");
  });
});
