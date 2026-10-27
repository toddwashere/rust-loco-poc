import { getDb } from "@/lib/db";
import { companies, contacts } from "@/lib/schema";
import { timed } from "@/lib/profiler";
import { eq, desc } from "drizzle-orm";


export async function GET(req: Request) {
  const page = Math.max(1, Number(new URL(req.url).searchParams.get("page") ?? 1));
  const { value, ms } = await timed("GET /api/contacts", () =>
    getDb().select({ id: contacts.id, first_name: contacts.firstName, last_name: contacts.lastName, email: contacts.email, company: companies.name })
      .from(contacts).leftJoin(companies, eq(contacts.companyId, companies.id))
      .orderBy(desc(contacts.id)).limit(20).offset((page - 1) * 20).all()
  );
  return Response.json(value, { headers: { "Server-Timing": `db;dur=${ms.toFixed(2)}` } });
}

export async function POST(req: Request) {
  const body = await req.json();
  if (!body.first_name || !body.email) return Response.json({ error: "first_name and email required" }, { status: 422 });
  const { value } = await timed("POST /api/contacts", () =>
    getDb().insert(contacts).values({ firstName: body.first_name, lastName: body.last_name ?? "", email: body.email, companyId: body.company_id ?? null }).returning().get()
  );
  return Response.json(value, { status: 201 });
}
