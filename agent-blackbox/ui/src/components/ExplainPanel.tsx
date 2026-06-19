import type { ExplainResponseV1 } from "../api/types";
import { useI18n } from "../i18n";
import { parseResolutionSteps } from "../utils/format";
import { DecisionChainView } from "./DecisionChainView";
import { RootCauseCard } from "./RootCauseCard";
import { SummaryCard } from "./SummaryCard";

interface ExplainPanelProps {
  data: ExplainResponseV1;
}

export function ExplainPanel({ data }: ExplainPanelProps) {
  const { t } = useI18n();
  const steps = parseResolutionSteps(data.resolution_flow);

  return (
    <div className="explain-grid">
      <SummaryCard summary={data.summary} />
      <RootCauseCard rootCause={data.root_cause} />

      <section className="card">
        <h2 className="card-title">{t.explainPanel.problemStatement}</h2>
        <p className="problem-text">{data.problem_statement}</p>
      </section>

      <DecisionChainView chain={data.decision_chain} />

      <section className="card">
        <h2 className="card-title">{t.explainPanel.resolutionFlow}</h2>
        {steps.length > 0 ? (
          <ol className="resolution-steps">
            {steps.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>
        ) : (
          <p className="summary-text">{data.resolution_flow}</p>
        )}
      </section>

      <p className="muted">
        {t.explain.schema}: {data.explain_schema_version} · {t.explain.schemaNote}
      </p>
    </div>
  );
}
