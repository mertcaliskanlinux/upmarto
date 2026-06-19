import { useEffect, useState } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { fetchExplain } from "../api/client";
import type { ExplainResponseV1 } from "../api/types";
import { ExplainPanel } from "../components/ExplainPanel";
import { Layout } from "../components/Layout";
import { useI18n } from "../i18n";

export function ExplainPage() {
  const { t } = useI18n();
  const { sessionId } = useParams<{ sessionId: string }>();
  const [searchParams] = useSearchParams();
  const eventId = searchParams.get("event_id") ?? undefined;

  const [data, setData] = useState<ExplainResponseV1 | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!sessionId) return;
    setLoading(true);
    setError(null);
    fetchExplain({ session_id: sessionId, event_id: eventId })
      .then(setData)
      .catch((err) =>
        setError(err instanceof Error ? err.message : t.explain.errorLoad),
      )
      .finally(() => setLoading(false));
  }, [sessionId, eventId, t.explain.errorLoad]);

  if (!sessionId) {
    return (
      <Layout>
        <p className="error-banner">{t.explain.missingSession}</p>
      </Layout>
    );
  }

  return (
    <Layout sessionId={sessionId}>
      <h2 className="page-title">{t.explain.title}</h2>
      <p className="muted" style={{ marginBottom: "0.75rem" }}>
        {t.explain.subtitle}{" "}
        <code className="session-id">{sessionId}</code>
        {eventId && (
          <>
            {" "}
            · {t.explain.scopedTo}{" "}
            <code className="session-id">{eventId.slice(0, 8)}…</code>
          </>
        )}
      </p>

      <div className="nav-actions">
        <Link to={`/timeline/${sessionId}`}>
          <button type="button">{t.explain.backTimeline}</button>
        </Link>
      </div>

      {error && <div className="error-banner">{error}</div>}
      {loading && <p className="loading">{t.explain.loading}</p>}
      {data && !loading && <ExplainPanel data={data} />}
    </Layout>
  );
}
