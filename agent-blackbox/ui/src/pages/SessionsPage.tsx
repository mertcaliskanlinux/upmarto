import { useCallback, useEffect, useState } from "react";
import { fetchProjectSessions } from "../api/client";
import type { Session } from "../api/types";
import { Layout } from "../components/Layout";
import { ProjectPicker } from "../components/ProjectPicker";
import { SessionList } from "../components/SessionList";
import { useI18n } from "../i18n";

const STORAGE_KEY = "upmarto_project_id";
const DEFAULT_PROJECT =
  import.meta.env.VITE_DEFAULT_PROJECT_ID ?? "agent-blackbox";

export function SessionsPage() {
  const { t } = useI18n();
  const [projectId, setProjectId] = useState(
    () => localStorage.getItem(STORAGE_KEY) ?? DEFAULT_PROJECT,
  );
  const [sessions, setSessions] = useState<Session[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);
    try {
      const data = await fetchProjectSessions(id);
      setSessions(data.sessions);
    } catch (err) {
      setError(err instanceof Error ? err.message : t.sessions.errorLoad);
      setSessions([]);
    } finally {
      setLoading(false);
    }
  }, [t.sessions.errorLoad]);

  useEffect(() => {
    load(projectId);
  }, [projectId, load]);

  function handleProjectChange(id: string) {
    localStorage.setItem(STORAGE_KEY, id);
    setProjectId(id);
  }

  return (
    <Layout>
      <h2 className="page-title">{t.sessions.title}</h2>
      <p className="muted" style={{ marginBottom: "1rem" }}>
        {t.sessions.subtitle}
      </p>

      <div className="toolbar sessions-toolbar">
        <ProjectPicker value={projectId} onChange={handleProjectChange} />
        <button type="button" onClick={() => load(projectId)} disabled={loading}>
          {t.sessions.refresh}
        </button>
      </div>

      {error && <div className="error-banner">{error}</div>}
      {loading ? (
        <p className="loading">{t.sessions.loading}</p>
      ) : (
        <SessionList sessions={sessions} projectId={projectId} />
      )}
    </Layout>
  );
}
