import { getDb } from "@/lib/db";
import { companies, contacts } from "@/lib/schema";
import { record } from "@/lib/profiler";
import { eq, desc } from "drizzle-orm";


export default async function ContactsPage({ searchParams }: { searchParams: Promise<{ page?: string }> }) {
  const t = performance.now();
  const page = Math.max(1, Number((await searchParams).page ?? 1));
  const rows = getDb()
    .select({ id: contacts.id, firstName: contacts.firstName, lastName: contacts.lastName, email: contacts.email, company: companies.name })
    .from(contacts).leftJoin(companies, eq(contacts.companyId, companies.id))
    .orderBy(desc(contacts.id)).limit(20).offset((page - 1) * 20).all();
  record("GET /contacts (data)", performance.now() - t);
  return (
    <main style={{ padding: 24, fontFamily: "system-ui" }}>
      <h1>Contacts</h1>
      <table>
        <thead><tr><th>Name</th><th>Email</th><th>Company</th></tr></thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r.id}><td>{r.firstName} {r.lastName}</td><td>{r.email}</td><td>{r.company}</td></tr>
          ))}
        </tbody>
      </table>
      <a href={`/contacts?page=${page + 1}`}>Next page →</a>
    </main>
  );
}

// Plain blocking SSR (no streaming shell) to match the Loco server-rendered page.
export const instant = false;
