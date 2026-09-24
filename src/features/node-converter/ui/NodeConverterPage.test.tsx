import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { ConvertNodeTextResult } from "../../../shared/contracts";
import { NodeConverterResult, NodeConverterStatus } from "./NodeConverterPage";

const fixture: ConvertNodeTextResult = {
  yaml: "proxies:\n  - type: wireguard\n    private-key: PLACEHOLDER_PRIVATE_KEY\n",
  templateYaml: null,
  templateFileName: null,
  nodeCount: 1,
  duplicateNodeCount: 0,
  compatibilityNormalizationCount: 0,
  warnings: [],
};

describe("NodeConverterResult", () => {
  it("offers the complete built-in template after a single IP node converts", () => {
    const markup = renderToStaticMarkup(<NodeConverterResult result={{
      ...fixture,
      templateYaml: "proxies:\n  - name: 192.0.2.10\n    private-key: PLACEHOLDER_PRIVATE_KEY\nproxy-groups: []\n",
      templateFileName: "192.0.2.10.yaml",
    }} />);

    expect(markup).toContain("内置模版");
    expect(markup).toContain("192.0.2.10.yaml");
    expect(markup).toContain("复制完整配置");
    expect(markup).toContain("proxy-groups");
  });

  it("renders sensitive YAML directly without a disclosure", () => {
    const markup = renderToStaticMarkup(<NodeConverterResult result={fixture} />);

    expect(markup).toContain("PLACEHOLDER_PRIVATE_KEY");
    expect(markup).toContain("复制节点 YAML");
    expect(markup).not.toContain("<details");
    expect(markup).not.toContain("<summary");
  });

  it("keeps sensitive YAML out of the live status summary", () => {
    const markup = renderToStaticMarkup(<NodeConverterStatus result={fixture} />);

    expect(markup).toContain('role="status"');
    expect(markup).toContain("1 个节点");
    expect(markup).not.toContain("PLACEHOLDER_PRIVATE_KEY");
    expect(markup).not.toContain("private-key");
  });
});
