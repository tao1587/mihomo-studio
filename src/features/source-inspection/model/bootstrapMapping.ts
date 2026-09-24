import type { BootstrapMappingInput } from "../../../shared/contracts";

export interface ParsedBootstrapMappings {
  mappings: BootstrapMappingInput[];
  error: string | null;
}

export function parseBootstrapMappings(value: string): ParsedBootstrapMappings {
  const lines = value.split(/\r\n|\r|\n/);
  const mappings: BootstrapMappingInput[] = [];
  const positions = new Set<string>();

  for (const [index, rawLine] of lines.entries()) {
    const line = rawLine.trim();
    if (!line) continue;
    const match = /^(\d+):(\d+)=(\d{1,3}(?:\.\d{1,3}){3})$/.exec(line);
    if (!match) return invalidLine(index + 1);

    const source = Number(match[1]);
    const node = Number(match[2]);
    const ipv4 = match[3];
    if (source < 1 || node < 1 || !isIpv4Literal(ipv4)) {
      return invalidLine(index + 1);
    }

    const position = `${source}:${node}`;
    if (positions.has(position)) {
      return {
        mappings: [],
        error: `第 ${index + 1} 行重复指定了 ${position}。`,
      };
    }
    positions.add(position);
    mappings.push({ source, node, ipv4 });
  }

  return { mappings, error: null };
}

function isIpv4Literal(value: string): boolean {
  const octets = value.split(".");
  return octets.length === 4
    && octets.every((octet) => /^\d{1,3}$/.test(octet) && Number(octet) <= 255);
}

function invalidLine(line: number): ParsedBootstrapMappings {
  return {
    mappings: [],
    error: `第 ${line} 行 bootstrap 映射格式无效；请使用 1:1=IPv4。`,
  };
}
