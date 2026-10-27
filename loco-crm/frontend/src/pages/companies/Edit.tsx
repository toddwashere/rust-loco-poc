import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { useCompany, useUpdateCompany } from "../../api/companies";
import type { UpdateCompany } from "../../bindings/UpdateCompany";

export function Edit() {
  const navigate = useNavigate();
  const { id: idParam } = useParams<{ id: string }>();
  const id = Number(idParam);
  const isValidId = Number.isFinite(id) && id > 0;

  const { data, isLoading, isError, error } = useCompany(isValidId ? id : NaN);
  const updateCompany = useUpdateCompany();

  const [form, setForm] = useState<UpdateCompany | null>(null);

  useEffect(() => {
    if (data) {
      setForm({
        name: data.name,
        domain: data.domain,
        industry: data.industry,
      });
    }
  }, [data]);

  if (!isValidId) {
    return <p role="alert">Invalid company id.</p>;
  }

  if (isLoading || !form) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!form) {
      return;
    }
    updateCompany.mutate(
      { id, data: form },
      {
        onSuccess: () => {
          navigate(`/companies/${id}`);
        },
      },
    );
  }

  return (
    <div>
      <h1>Edit Company</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="name">Name</label>
          <input
            id="name"
            type="text"
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="domain">Domain</label>
          <input
            id="domain"
            type="text"
            value={form.domain}
            onChange={(e) => setForm({ ...form, domain: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="industry">Industry</label>
          <input
            id="industry"
            type="text"
            value={form.industry}
            onChange={(e) => setForm({ ...form, industry: e.target.value })}
          />
        </div>
        <button type="submit" disabled={updateCompany.isPending}>
          {updateCompany.isPending ? "Saving…" : "Save"}
        </button>
        {updateCompany.error && <p role="alert">{updateCompany.error.message}</p>}
      </form>
    </div>
  );
}
