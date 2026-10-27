import { connection } from "next/server";
import { snapshot } from "@/lib/profiler";

export default async function ProfilerPage() {
  await connection();
  const s = snapshot();
  return (
    <main style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1>Profiler</h1>
      <p>RSS: {s.memory_rss_mb} MB</p>
      <table>
        <thead><tr><th>Route</th><th>Count</th><th>p50 ms</th><th>p95 ms</th><th>p99 ms</th></tr></thead>
        <tbody>{s.routes.map((r) => (<tr key={r.route}><td>{r.route}</td><td>{r.count}</td><td>{r.p50_ms}</td><td>{r.p95_ms}</td><td>{r.p99_ms}</td></tr>))}</tbody>
      </table>
    </main>
  );
}

// Plain blocking SSR (no streaming shell) to match the Loco server-rendered page.
export const instant = false;
