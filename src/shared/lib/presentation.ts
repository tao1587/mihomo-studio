export function formatSourceLabel(format: string) {
  const labels: Record<string, string> = {
    "mihomo-yaml": "Mihomo YAML",
    "mihomo-provider-yaml": "Provider YAML",
    "uri-list": "URI 列表",
    "base64-uri-list": "Base64 URI",
    "base64-mihomo-yaml": "Base64 YAML",
    unknown: "待 resolver",
  };
  return labels[format] || format;
}

export function safeErrorMessage(error: unknown, fallback: string) {
  return typeof error === "string" && error.length <= 500 ? error : fallback;
}
