export function CountdownRing() {
  return (
    <svg className="size-9 -rotate-90" viewBox="0 0 40 40" aria-hidden="true">
      <circle cx="20" cy="20" r="16" fill="none" stroke="rgba(255,255,255,.1)" strokeWidth="3" />
      <circle
        className="vox-countdown-ring"
        cx="20"
        cy="20"
        r="16"
        fill="none"
        stroke="#ff718d"
        strokeLinecap="round"
        strokeWidth="3"
        pathLength="100"
      />
    </svg>
  );
}
