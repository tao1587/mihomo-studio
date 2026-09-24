import { describe, expect, it } from "vitest";

import { countConverterInputLines } from "./input";

describe("countConverterInputLines", () => {
  it("counts non-empty node input lines", () => {
    expect(countConverterInputLines("vless://fixture-a\n\nvless://fixture-b")).toBe(2);
  });

  it("counts multi-line WireGuard configuration fields", () => {
    const input = [
      "[Interface]",
      "PrivateKey = placeholder",
      "Address = 192.0.2.1/32",
      "",
      "[Peer]",
      "PublicKey = placeholder",
      "Endpoint = wireguard.example.invalid:51820",
      "AllowedIPs = 0.0.0.0/0",
    ].join("\n");

    expect(countConverterInputLines(input)).toBe(7);
  });

  it("supports CRLF and ignores whitespace-only lines", () => {
    expect(countConverterInputLines("vless://fixture-a\r\n   \r\nvless://fixture-b\r")).toBe(2);
  });
});
