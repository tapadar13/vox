import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Toggle } from "./Toggle";

describe("Toggle", () => {
  it("reports the next checked state", () => {
    const onChange = vi.fn();
    render(
      <Toggle
        checked={false}
        onChange={onChange}
        label="Paste at the cursor"
        description="Uses Accessibility access."
      />,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Paste at the cursor" }));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
