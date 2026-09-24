import { describe, expect, it } from "vitest";

import { countNonEmptyNodeLines } from "./directNodeInput";

describe("countNonEmptyNodeLines", () => {
  it("counts non-empty node lines across common newline formats", () => {
    expect(countNonEmptyNodeLines("vless://one\n\n trojan://two \r\nss://three\r")).toBe(3);
  });

  it("returns zero for empty or whitespace-only input", () => {
    expect(countNonEmptyNodeLines("  \n\t\r\n")).toBe(0);
  });
});
