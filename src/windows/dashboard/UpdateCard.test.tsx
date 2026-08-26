import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { UpdateCard } from "./UpdateCard";

const updater = vi.hoisted(() => ({
  checkForUpdate: vi.fn(),
  installUpdate: vi.fn(),
}));

vi.mock("../../lib/updater", () => updater);

describe("UpdateCard", () => {
  beforeEach(() => vi.clearAllMocks());

  it("reports when Vox is current", async () => {
    updater.checkForUpdate.mockResolvedValue(null);
    render(<UpdateCard />);

    fireEvent.click(screen.getByRole("button", { name: "Check now" }));

    expect(await screen.findByText("v0.1.0 · Up to date")).toBeInTheDocument();
  });

  it("installs an available update", async () => {
    const candidate = {
      version: "0.2.0",
      close: vi.fn().mockResolvedValue(undefined),
    };
    updater.checkForUpdate.mockResolvedValue(candidate);
    updater.installUpdate.mockResolvedValue(undefined);
    render(<UpdateCard />);

    fireEvent.click(screen.getByRole("button", { name: "Check now" }));
    expect(await screen.findByText("Vox 0.2.0 · Install")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Install & restart" }));

    await waitFor(() => expect(updater.installUpdate).toHaveBeenCalledWith(candidate, expect.any(Function)));
  });
});
