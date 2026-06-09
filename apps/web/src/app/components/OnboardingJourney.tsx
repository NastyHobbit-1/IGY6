

export function OnboardingJourney({
  sourceCount,
  evidenceCount,
  chunkCount,
  llmEnabled,
  llmModel
}: {
  sourceCount: number;
  evidenceCount: number;
  chunkCount: number;
  llmEnabled: boolean;
  llmModel: string;
}) {
  const steps = [
    {
      id: "chat",
      title: "1 · Chat",
      detail: "Ask questions, run safe actions, open Data/Work/Settings by typing here.",
      state: "ready" as const
    },
    {
      id: "data",
      title: "2 · Add data",
      detail: sourceCount > 0 ? `${sourceCount} source(s) registered — upload or import more anytime.` : "No sources yet — say \"add data\" in chat or open the Data tab.",
      state: sourceCount > 0 ? ("ready" as const) : ("next" as const)
    },
    {
      id: "evidence",
      title: "3 · Evidence",
      detail: chunkCount > 0 ? `${chunkCount} chunks indexed — Ollama can summarize with citations.` : "Upload text first, then check processing before asking evidence questions.",
      state: evidenceCount > 0 || chunkCount > 0 ? ("ready" as const) : ("waiting" as const)
    },
    {
      id: "llm",
      title: "4 · Local model",
      detail: llmEnabled ? `Ollama active (${llmModel}).` : "Install Ollama or run igy6 start while Ollama is running to auto-enable.",
      state: llmEnabled ? ("ready" as const) : ("optional" as const)
    }
  ];

  return (
    <section className="journeyStrip" aria-label="Getting started">
      {steps.map((step) => (
        <article className={`journeyCard journey-${step.state}`} key={step.id}>
          <strong>{step.title}</strong>
          <span>{step.detail}</span>
        </article>
      ))}
    </section>
  );
}

