import type { Event } from "../api/types";
import { useI18n } from "../i18n";
import { EventItem } from "./EventItem";

interface TimelineViewProps {
  events: Event[];
  selectedEventId?: string;
  onSelectEvent?: (event: Event) => void;
}

export function TimelineView({ events, selectedEventId, onSelectEvent }: TimelineViewProps) {
  const { t } = useI18n();

  if (events.length === 0) {
    return <p className="muted">{t.timeline.noEvents}</p>;
  }

  return (
    <ul className="event-list">
      {events.map((event) => (
        <EventItem
          key={event.id}
          event={event}
          selected={event.id === selectedEventId}
          onSelect={onSelectEvent}
        />
      ))}
    </ul>
  );
}
