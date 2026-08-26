import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { LiveTranscript } from "./LiveTranscript";

describe("LiveTranscript", () => {
  it("separates stable words from the provisional tail", () => {
    const { container } = render(
      <LiveTranscript text="the stable words are changing" stableWords={3} />,
    );
    expect(screen.getByLabelText("Live transcript: the stable words are changing")).toBeVisible();
    const spans = container.querySelectorAll("span");
    expect(spans[0]).toHaveTextContent("the stable words");
    expect(spans[1]).toHaveTextContent("are changing");
  });

  it("handles a fully stable transcript", () => {
    const { container } = render(<LiveTranscript text="finished phrase" stableWords={2} />);
    expect(container.querySelectorAll("span")).toHaveLength(1);
    expect(container).toHaveTextContent("finished phrase");
  });
});
