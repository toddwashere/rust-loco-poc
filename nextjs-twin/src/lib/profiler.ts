// Minimal in-process profiler mirroring the Loco one: per-route latency samples + Server-Timing.
type Stats = { samples: number[]; count: number };
const g = globalThis as unknown as { __prof?: Map<string, Stats> };
const routes = (g.__prof ??= new Map());
const MAX = 1000;

export function record(route: string, ms: number) {
  const s = routes.get(route) ?? { samples: [], count: 0 };
  s.count++;
  s.samples.push(ms);
  if (s.samples.length > MAX) s.samples.shift();
  routes.set(route, s);
}

export async function timed<T>(route: string, fn: () => T | Promise<T>): Promise<{ value: T; ms: number }> {
  const t = performance.now();
  const value = await fn();
  const ms = performance.now() - t;
  record(route, ms);
  return { value, ms };
}

const pct = (sorted: number[], p: number) => sorted[Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))] ?? 0;

export function snapshot() {
  return {
    memory_rss_mb: +(process.memoryUsage().rss / 1048576).toFixed(1),
    routes: [...routes.entries()].map(([route, s]) => {
      const sorted = [...s.samples].sort((a, b) => a - b);
      return { route, count: s.count, p50_ms: +pct(sorted, 50).toFixed(3), p95_ms: +pct(sorted, 95).toFixed(3), p99_ms: +pct(sorted, 99).toFixed(3) };
    }),
  };
}
