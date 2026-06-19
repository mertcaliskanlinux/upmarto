import { useI18n } from "../i18n";

interface DecisionChainViewProps {
  chain: string[];
}

export function DecisionChainView({ chain }: DecisionChainViewProps) {
  const { t } = useI18n();

  return (
    <section className="card">
      <h2 className="card-title">{t.explainPanel.decisionChain}</h2>
      {chain.length === 0 ? (
        <p className="muted">{t.explainPanel.noChain}</p>
      ) : (
        <ol className="decision-chain">
          {chain.map((step, i) => (
            <li key={i}>{step}</li>
          ))}
        </ol>
      )}
    </section>
  );
}
