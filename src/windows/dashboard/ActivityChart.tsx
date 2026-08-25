import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis } from "recharts";

import type { ActivityDay } from "../../lib/types";

interface ActivityChartProps {
  activity: ActivityDay[];
}

export function ActivityChart({ activity }: ActivityChartProps) {
  const recent = activity.slice(-28).map((day) => ({
    ...day,
    label: new Date(`${day.date}T12:00:00`).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
    }),
  }));

  return (
    <div className="h-36 w-full" aria-label="Words dictated over the last four weeks">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={recent} margin={{ top: 8, right: 2, bottom: 0, left: 2 }}>
          <defs>
            <linearGradient id="voxActivity" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="#8b79ff" stopOpacity={0.52} />
              <stop offset="100%" stopColor="#8b79ff" stopOpacity={0} />
            </linearGradient>
          </defs>
          <XAxis
            dataKey="label"
            axisLine={false}
            tickLine={false}
            minTickGap={30}
            tick={{ fill: "rgba(255,255,255,.3)", fontSize: 9 }}
          />
          <Tooltip
            cursor={{ stroke: "rgba(255,255,255,.12)" }}
            contentStyle={{
              background: "rgba(13,15,24,.95)",
              border: "1px solid rgba(255,255,255,.1)",
              borderRadius: 12,
              color: "white",
              fontSize: 11,
            }}
            formatter={(value) => [`${Number(value).toLocaleString()} words`, "Dictated"]}
            labelStyle={{ color: "rgba(255,255,255,.45)", marginBottom: 4 }}
          />
          <Area
            type="monotone"
            dataKey="words"
            stroke="#9b8cff"
            strokeWidth={2}
            fill="url(#voxActivity)"
            activeDot={{ r: 3, strokeWidth: 0, fill: "#c0b8ff" }}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
