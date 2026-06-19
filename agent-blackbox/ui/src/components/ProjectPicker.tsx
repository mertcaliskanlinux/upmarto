import { useEffect, useMemo, useState } from "react";

import { fetchProjects } from "../api/client";
import { useI18n } from "../i18n";

const CUSTOM_VALUE = "__custom__";

interface ProjectPickerProps {
  value: string;
  onChange: (projectId: string) => void;
}

function mergeProjects(
  remote: string[],
  current: string,
  fallback: string,
): string[] {
  const set = new Set<string>();
  for (const id of remote) {
    if (id.trim()) set.add(id);
  }
  if (current.trim()) set.add(current.trim());
  if (fallback.trim()) set.add(fallback.trim());
  return Array.from(set).sort((a, b) => a.localeCompare(b));
}

export function ProjectPicker({ value, onChange }: ProjectPickerProps) {
  const { t } = useI18n();
  const defaultProject =
    import.meta.env.VITE_DEFAULT_PROJECT_ID?.trim() ?? "agent-blackbox";

  const [remoteProjects, setRemoteProjects] = useState<string[]>([]);
  const [loadError, setLoadError] = useState(false);
  const [mode, setMode] = useState<"select" | "custom">(() =>
    value ? "select" : "select",
  );
  const [customValue, setCustomValue] = useState(value);

  const options = useMemo(
    () => mergeProjects(remoteProjects, value, defaultProject),
    [remoteProjects, value, defaultProject],
  );

  const selectValue =
    mode === "custom" || !options.includes(value) ? CUSTOM_VALUE : value;

  useEffect(() => {
    let cancelled = false;
    fetchProjects()
      .then((data) => {
        if (!cancelled) {
          setRemoteProjects(data.projects);
          setLoadError(false);
        }
      })
      .catch(() => {
        if (!cancelled) setLoadError(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function handleSelectChange(next: string) {
    if (next === CUSTOM_VALUE) {
      setMode("custom");
      setCustomValue(value);
      return;
    }
    setMode("select");
    onChange(next);
  }

  function handleCustomSubmit(e: React.FormEvent) {
    e.preventDefault();
    const trimmed = customValue.trim();
    if (!trimmed) return;
    setMode("select");
    onChange(trimmed);
  }

  return (
    <div className="project-picker">
      <label className="project-picker-label">
        {t.sessions.project}
        <select
          className="project-select"
          value={selectValue}
          onChange={(e) => handleSelectChange(e.target.value)}
          aria-label={t.sessions.project}
        >
          {options.length === 0 && (
            <option value="" disabled>
              {t.sessions.noProjects}
            </option>
          )}
          {options.map((id) => (
            <option key={id} value={id}>
              {id}
            </option>
          ))}
          <option value={CUSTOM_VALUE}>{t.sessions.customProject}</option>
        </select>
      </label>

      {mode === "custom" && (
        <form className="project-custom" onSubmit={handleCustomSubmit}>
          <input
            value={customValue}
            onChange={(e) => setCustomValue(e.target.value)}
            placeholder={t.sessions.projectPlaceholder}
            autoFocus
          />
          <button type="submit">{t.sessions.apply}</button>
        </form>
      )}

      {loadError && (
        <p className="muted project-picker-hint">{t.sessions.projectsOffline}</p>
      )}
    </div>
  );
}
