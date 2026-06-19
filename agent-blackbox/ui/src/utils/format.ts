import type { EventType } from "../api/types";
import type { TranslationKeys } from "../i18n/translations";

export function formatTimestamp(ms: number, locale?: string): string {
  return new Date(ms).toLocaleString(locale, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function formatTimeRange(start: number, end: number | null, locale?: string): string {
  if (end == null) return formatTimestamp(start, locale);
  return `${formatTimestamp(start, locale)} → ${formatTimestamp(end, locale)}`;
}

export function payloadPreview(payload: Record<string, unknown>): string {
  const parts: string[] = [];
  for (const [key, value] of Object.entries(payload)) {
    const text =
      typeof value === "string" ? value : JSON.stringify(value);
    const clipped = text.length > 80 ? `${text.slice(0, 80)}…` : text;
    parts.push(`${key}: ${clipped}`);
  }
  return parts.join(" · ") || "—";
}

export function eventTypeLabel(
  type: EventType,
  labels: TranslationKeys["eventTypes"],
): string {
  return labels[type] ?? type.replace(/_/g, " ");
}

export function parseResolutionSteps(flow: string): string[] {
  const trimmed = flow.trim();
  if (!trimmed) return [];
  return trimmed
    .split(/\.\s+(?=\d+\.)/)
    .map((s) => s.replace(/^\d+\.\s*/, "").replace(/\.$/, "").trim())
    .filter(Boolean);
}
