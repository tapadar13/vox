export function CountdownRing() {
  return (
    <span className="relative grid size-7 shrink-0 place-items-center rounded-full bg-[conic-gradient(#f4d37c,#ff4d8d_52%,rgba(255,255,255,.12)_100%)]">
      <svg className="absolute size-7 -rotate-90" viewBox="0 0 40 40" aria-hidden="true">
      <circle
        className="vox-countdown-ring"
        cx="20"
        cy="20"
        r="18"
        fill="none"
        stroke="rgba(255,255,255,.32)"
        strokeLinecap="round"
        strokeWidth="2"
        pathLength="100"
      />
      </svg>
      <span className="grid size-[22px] place-items-center rounded-full bg-[#171920] text-[15px] leading-[15px] text-white">×</span>
    </span>
  );
}
