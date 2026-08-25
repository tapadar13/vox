import { lazy, Suspense } from "react";

const Dashboard = lazy(() =>
  import("./windows/dashboard/Dashboard").then(({ Dashboard: component }) => ({ default: component })),
);
const Pill = lazy(() =>
  import("./windows/pill/Pill").then(({ Pill: component }) => ({ default: component })),
);

export function App() {
  const windowName = new URLSearchParams(window.location.search).get("window");
  return (
    <Suspense fallback={<main className="min-h-screen bg-transparent" />}>
      {windowName === "pill" ? <Pill /> : <Dashboard />}
    </Suspense>
  );
}
