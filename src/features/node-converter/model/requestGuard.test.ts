import { describe, expect, it } from "vitest";

import { ConversionRequestGuard } from "./requestGuard";

describe("ConversionRequestGuard", () => {
  it("rejects a response after the input invalidates its request", () => {
    const guard = new ConversionRequestGuard();
    const staleRequest = guard.begin();

    guard.invalidate();

    expect(guard.isCurrent(staleRequest)).toBe(false);
  });

  it("keeps only the newest overlapping conversion response", () => {
    const guard = new ConversionRequestGuard();
    const firstRequest = guard.begin();
    const secondRequest = guard.begin();

    expect(guard.isCurrent(firstRequest)).toBe(false);
    expect(guard.isCurrent(secondRequest)).toBe(true);
  });
});
