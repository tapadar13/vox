import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { Button } from "./Button";

describe("Button", () => {
  it("blocks duplicate actions while busy", () => {
    const onClick = vi.fn();
    render(<Button busy onClick={onClick}>Download</Button>);
    const button = screen.getByRole("button", { name: "Download" });
    expect(button).toBeDisabled();
    fireEvent.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });
});
