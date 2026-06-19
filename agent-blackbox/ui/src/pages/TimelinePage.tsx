import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { fetchTimeline } from "../api/client";
import type { Event } from "../api/types";
import { Layout } from "../components/Layout";
import { TimelineView } from "../components/TimelineView";
import { useI18n } from "../i18n";

export function TimelinePage() {
  const { t } = useI18n();
  const { sessionId } = useParams<{ sessionId: string }>();
  const navigate = useNavigate();
  const [events, setEvents] = useState<Event[]>([]);
  const [selected, setSelected] = useState<Event | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!sessionId) return;
    setLoading(true);
    setError(null);
    fetchTimeline(sessionId)
      .then((data) => setEvents(data.events))
      .catch((err) =>
        setError(err instanceof Error ? err.message : t.timeline.errorLoad),
      )
      .finally(() => setLoading(false));
  }, [sessionId, t.timeline.errorLoad]);

  if (!sessionId) {
    return (
      <Layout>
        <p className="error-banner">{t.timeline.missingSession}</p>
      </Layout>
    );
  }

  return (
    <Layout sessionId={sessionId}>
      <h2 className="page-title">{t.timeline.title}</h2>
      <p className="muted" style={{ marginBottom: "0.75rem" }}>
        {t.timeline.sessionLabel}{" "}
        <code className="session-id">{sessionId}</code> · {events.length}{" "}
        {t.timeline.eventsCount}
      </p>

      <div className="nav-actions">
        <button type="button" onClick={() => navigate("/sessions")}>
          {t.timeline.backSessions}
        </button>
        <Link to={`/explain/${sessionId}`}>
          <button type="button">{t.timeline.whyButton}</button>
        </Link>
        {selected && (
          <Link to={`/explain/${sessionId}?event_id=${selected.id}`}>
            <button type="button">{t.timeline.explainSelected}</button>
          </Link>
        )}
      </div>

      {error && <div className="error-banner">{error}</div>}
      {loading ? (
        <p className="loading">{t.timeline.loading}</p>
      ) : (
        <TimelineView
          events={events}
          selectedEventId={selected?.id}
          onSelectEvent={setSelected}
        />
      )}
    </Layout>
  );
}
