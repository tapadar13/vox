interface LiveTranscriptProps {
  text: string;
  stableWords: number;
}

export function LiveTranscript({ text, stableWords }: LiveTranscriptProps) {
  const words = text.trim().split(/\s+/).filter(Boolean);
  const stable = words.slice(0, stableWords).join(" ");
  const provisional = words.slice(stableWords).join(" ");

  return (
    <p className="truncate text-[10px] leading-4" aria-label={`Live transcript: ${text}`}>
      {stable && <span className="text-white/58">{stable}</span>}
      {stable && provisional && " "}
      {provisional && <span className="text-white/28">{provisional}</span>}
    </p>
  );
}
