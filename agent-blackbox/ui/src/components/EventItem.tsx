import type { Event, EventType } from "../api/types";
import { useI18n } from "../i18n";
import { eventTypeLabel, formatTimestamp, payloadPreview } from "../utils/format";

interface EventItemProps {
  event: Event;
  selected?: boolean;
  onSelect?: (event: Event) => void;
}

function badgeClass(type: EventType): string {
  if (type === "test_failed") return "badge test-fail";
  if (type === "test_run" || type === "test_passed") return "badge test";
  if (type.startsWith("file")) return "badge file";
  if (type === "command_executed") return "badge command";
  if (type === "git_commit") return "badge git";
  if (type === "agent_message") return "badge agent";
  return "badge";
}

export function EventItem({ event, selected, onSelect }: EventItemProps) {
  const { t, bcp47 } = useI18n();

  return (
    <li
      className={`event-item${selected ? " selected" : ""}`}
      onClick={() => onSelect?.(event)}
    >
      <div className="event-item-header">
        <span className={badgeClass(event.event_type)}>
          {eventTypeLabel(event.event_type, t.eventTypes)}
        </span>
        <span className="event-item-time">{formatTimestamp(event.timestamp, bcp47)}</span>
      </div>
      <div className="event-item-preview">{payloadPreview(event.payload)}</div>
    </li>
  );
}
