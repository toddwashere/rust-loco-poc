import { useState } from "react";
import { useNavigate } from "react-router";
import { useCreateDeal } from "../../api/deals";
import type { CreateDeal } from "../../bindings/CreateDeal";

export function New() {
  const navigate = useNavigate();
  const createDeal = useCreateDeal();

  const [form, setForm] = useState<CreateDeal>({
    title: "",
    amount_cents: null,
    stage: "",
    company_id: null,
    contact_id: null,
  });

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    createDeal.mutate(form, {
      onSuccess: (created) => {
        navigate(`/deals/${created.id}`);
      },
    });
  }

  return (
    <div>
      <h1>New Deal</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="title">Title</label>
          <input
            id="title"
            type="text"
            value={form.title ?? ""}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="amount_cents">Amount cents</label>
          <input
            id="amount_cents"
            type="number"
            value={form.amount_cents ?? ""}
            onChange={(e) =>
              setForm({ ...form, amount_cents: e.target.value === "" ? null : Number(e.target.value) })
            }
          />
        </div>
        <div>
          <label htmlFor="stage">Stage</label>
          <input
            id="stage"
            type="text"
            value={form.stage ?? ""}
            onChange={(e) => setForm({ ...form, stage: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="company_id">Company id</label>
          <input
            id="company_id"
            type="number"
            value={form.company_id ?? ""}
            onChange={(e) =>
              setForm({ ...form, company_id: e.target.value === "" ? null : Number(e.target.value) })
            }
          />
        </div>
        <div>
          <label htmlFor="contact_id">Contact id</label>
          <input
            id="contact_id"
            type="number"
            value={form.contact_id ?? ""}
            onChange={(e) =>
              setForm({ ...form, contact_id: e.target.value === "" ? null : Number(e.target.value) })
            }
          />
        </div>
        <button type="submit" disabled={createDeal.isPending}>
          {createDeal.isPending ? "Creating…" : "Create"}
        </button>
        {createDeal.error && <p role="alert">{createDeal.error.message}</p>}
      </form>
    </div>
  );
}
