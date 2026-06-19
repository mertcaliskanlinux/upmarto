import { useI18n } from "../i18n";

interface RootCauseCardProps {
  rootCause: string;
}

export function RootCauseCard({ rootCause }: RootCauseCardProps) {
  const { t } = useI18n();

  return (
    <section className="card">
      <h2 className="card-title">{t.explainPanel.rootCause}</h2>
      <p className="root-cause-text">{rootCause}</p>
    </section>
  );
}
