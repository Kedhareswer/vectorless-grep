export function formatLatency(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  return `${(ms / 1000).toFixed(2)}s`;
}

export function formatConfidence(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function nodeTypeLabel(value: string): string {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

export function nodeIcon(nodeType: string): string {
  switch (nodeType.toLowerCase()) {
    case "document":   return "\u{1F4C4}";
    case "section":    return "\u{1F4C1}";
    case "subsection": return "\u{1F4D1}";
    case "table":      return "\u{1F5C3}";
    case "figure":     return "\u{1F5BC}";
    case "equation":   return "\u2211";
    case "paragraph":  return "\u{1F4C4}";
    case "claim":      return "\u2605";
    case "caption":    return "\u2014";
    case "reference":  return "\u2197";
    default:           return "\u2022";
  }
}

export function depthForOrdinal(ordinalPath: string): number {
  if (ordinalPath === "root") return 0;
  return ordinalPath.split(".").length - 1;
}
