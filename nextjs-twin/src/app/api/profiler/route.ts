import { connection } from "next/server";
import { snapshot } from "@/lib/profiler";
export async function GET() {
  await connection();
  return Response.json(snapshot());
}
