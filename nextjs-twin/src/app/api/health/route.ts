import { connection } from "next/server";

export async function GET() {
  await connection(); // force per-request execution so the benchmark measures the runtime, not a static file
  return Response.json({ ok: true });
}
