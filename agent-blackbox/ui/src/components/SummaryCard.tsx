import { useI18n } from "../i18n";

interface SummaryCardProps {
  summary: string;
}

export function SummaryCard({ summary }: SummaryCardProps) {
  const { t } = useI18n();

  return (
    <section className="card">
      <h2 className="card-title">{t.explainPanel.summary}</h2>
      <p className="summary-text">{summary}</p>
    </section>
  );
}
