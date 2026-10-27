import { useState } from "react";
import { useNavigate } from "react-router";
import { useCreateCompany } from "../../api/companies";
import type { CreateCompany } from "../../bindings/CreateCompany";

export function New() {
  const navigate = useNavigate();
  const createCompany = useCreateCompany();

  const [form, setForm] = useState<CreateCompany>({
    name: "",
    domain: null,
    industry: null,
  });

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    createCompany.mutate(form, {
      onSuccess: (created) => {
        navigate(`/companies/${created.id}`);
      },
    });
  }

  return (
    <div>
      <h1>New Company</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="name">Name</label>
          <input
            id="name"
            type="text"
            value={form.name ?? ""}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="domain">Domain</label>
          <input
            id="domain"
            type="text"
            value={form.domain ?? ""}
            onChange={(e) => setForm({ ...form, domain: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="industry">Industry</label>
          <input
            id="industry"
            type="text"
            value={form.industry ?? ""}
            onChange={(e) => setForm({ ...form, industry: e.target.value })}
          />
        </div>
        <button type="submit" disabled={createCompany.isPending}>
          {createCompany.isPending ? "Creating…" : "Create"}
        </button>
        {createCompany.error && <p role="alert">{createCompany.error.message}</p>}
      </form>
    </div>
  );
}
