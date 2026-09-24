import { describe, expect, it } from "vitest";

import { parseBootstrapMappings } from "./bootstrapMapping";

describe("parseBootstrapMappings", () => {
  it("parses one-based source and node positions", () => {
    expect(parseBootstrapMappings("1:1=192.0.2.10\n2:3=198.51.100.20")).toEqual({
      mappings: [
        { source: 1, node: 1, ipv4: "192.0.2.10" },
        { source: 2, node: 3, ipv4: "198.51.100.20" },
      ],
      error: null,
    });
  });

  it("rejects invalid, zero-based, and duplicate positions", () => {
    expect(parseBootstrapMappings("1:1=999.0.2.10").error).toContain("第 1 行");
    expect(parseBootstrapMappings("0:1=192.0.2.10").error).toContain("第 1 行");
    expect(parseBootstrapMappings("1:1=192.0.2.10\n1:1=192.0.2.11").error)
      .toContain("重复指定");
  });

  it("treats blank input as no mappings", () => {
    expect(parseBootstrapMappings(" \n\t")).toEqual({ mappings: [], error: null });
  });
});
