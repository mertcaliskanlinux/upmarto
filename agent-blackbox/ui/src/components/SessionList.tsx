import { useNavigate } from "react-router-dom";
import type { Session } from "../api/types";
import { useI18n } from "../i18n";
import { formatTimeRange } from "../utils/format";

interface SessionListProps {
  sessions: Session[];
  projectId: string;
}

export function SessionList({ sessions, projectId }: SessionListProps) {
  const navigate = useNavigate();
  const { t, bcp47 } = useI18n();

  if (sessions.length === 0) {
    return (
      <p className="muted">
        {t.sessions.empty} <code>{projectId}</code>.
      </p>
    );
  }

  return (
    <table className="session-table">
      <thead>
        <tr>
          <th>{t.sessionList.session}</th>
          <th>{t.sessionList.project}</th>
          <th>{t.sessionList.events}</th>
          <th>{t.sessionList.timeRange}</th>
        </tr>
      </thead>
      <tbody>
        {sessions.map((s) => (
          <tr key={s.id} onClick={() => navigate(`/timeline/${s.id}`)}>
            <td className="session-id">{s.id}</td>
            <td>{s.project_id}</td>
            <td>{s.event_count}</td>
            <td className="muted">{formatTimeRange(s.started_at, s.ended_at, bcp47)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
