import { describe, expect, it } from "vitest";

import { advanceUpdateProgress, emptyUpdateProgress } from "./updateProgress";

describe("advanceUpdateProgress", () => {
  it("calculates bounded progress from streamed chunks", () => {
    const started = advanceUpdateProgress(emptyUpdateProgress, {
      event: "Started",
      data: { contentLength: 1_000 },
    });
    const halfway = advanceUpdateProgress(started, {
      event: "Progress",
      data: { chunkLength: 500 },
    });
    const overReported = advanceUpdateProgress(halfway, {
      event: "Progress",
      data: { chunkLength: 700 },
    });

    expect(halfway).toMatchObject({ downloadedBytes: 500, percent: 50 });
    expect(overReported.percent).toBe(100);
  });

  it("supports servers that omit content length", () => {
    const started = advanceUpdateProgress(emptyUpdateProgress, {
      event: "Started",
      data: {},
    });
    const finished = advanceUpdateProgress(started, { event: "Finished" });

    expect(started.percent).toBeUndefined();
    expect(finished).toMatchObject({ finished: true, percent: undefined });
  });
});
