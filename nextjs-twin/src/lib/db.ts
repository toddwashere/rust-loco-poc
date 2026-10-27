import Database from "better-sqlite3";
import { drizzle } from "drizzle-orm/better-sqlite3";
import * as schema from "./schema";

// Same shape and seed size as the Loco app (50 companies / 1000 contacts) so benchmarks compare like for like.
function open() {
  const sqlite = new Database(process.env.DB_PATH ?? "twin.sqlite");
  sqlite.pragma("journal_mode = WAL");
  sqlite.exec(`
    CREATE TABLE IF NOT EXISTS companies (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, domain TEXT);
    CREATE TABLE IF NOT EXISTS contacts (id INTEGER PRIMARY KEY AUTOINCREMENT, first_name TEXT NOT NULL,
      last_name TEXT NOT NULL, email TEXT NOT NULL, company_id INTEGER REFERENCES companies(id));
  `);
  const { n } = sqlite.prepare("SELECT COUNT(*) AS n FROM companies").get() as { n: number };
  if (n === 0) {
    const insCo = sqlite.prepare("INSERT INTO companies (name, domain) VALUES (?, ?)");
    const insCt = sqlite.prepare("INSERT INTO contacts (first_name, last_name, email, company_id) VALUES (?, ?, ?, ?)");
    sqlite.transaction(() => {
      for (let i = 1; i <= 50; i++) insCo.run(`Company ${i}`, `company${i}.example`);
      for (let i = 1; i <= 1000; i++) insCt.run(`First${i}`, `Last${i}`, `person${i}@example.com`, ((i - 1) % 50) + 1);
    })();
  }
  return drizzle(sqlite, { schema });
}

const g = globalThis as unknown as { __db?: ReturnType<typeof open> };
// Lazy: Next imports route modules at build time, so opening here would lock the file across build workers.
export const getDb = () => (g.__db ??= open());
