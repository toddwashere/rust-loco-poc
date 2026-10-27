import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { useDeal, useUpdateDeal } from "../../api/deals";
import type { UpdateDeal } from "../../bindings/UpdateDeal";

export function Edit() {
  const navigate = useNavigate();
  const { id: idParam } = useParams<{ id: string }>();
  const id = Number(idParam);
  const isValidId = Number.isFinite(id) && id > 0;

  const { data, isLoading, isError, error } = useDeal(isValidId ? id : NaN);
  const updateDeal = useUpdateDeal();

  const [form, setForm] = useState<UpdateDeal | null>(null);

  useEffect(() => {
    if (data) {
      setForm({
        title: data.title,
        amount_cents: data.amount_cents,
        stage: data.stage,
        company_id: data.company_id,
        contact_id: data.contact_id,
      });
    }
  }, [data]);

  if (!isValidId) {
    return <p role="alert">Invalid deal id.</p>;
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
    updateDeal.mutate(
      { id, data: form },
      {
        onSuccess: () => {
          navigate(`/deals/${id}`);
        },
      },
    );
  }

  return (
    <div>
      <h1>Edit Deal</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="title">Title</label>
          <input
            id="title"
            type="text"
            value={form.title}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="amount_cents">Amount cents</label>
          <input
            id="amount_cents"
            type="number"
            value={form.amount_cents}
            onChange={(e) =>
              setForm({ ...form, amount_cents: Number(e.target.value) })
            }
          />
        </div>
        <div>
          <label htmlFor="stage">Stage</label>
          <input
            id="stage"
            type="text"
            value={form.stage}
            onChange={(e) => setForm({ ...form, stage: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="company_id">Company id</label>
          <input
            id="company_id"
            type="number"
            value={form.company_id}
            onChange={(e) =>
              setForm({ ...form, company_id: Number(e.target.value) })
            }
          />
        </div>
        <div>
          <label htmlFor="contact_id">Contact id</label>
          <input
            id="contact_id"
            type="number"
            value={form.contact_id}
            onChange={(e) =>
              setForm({ ...form, contact_id: Number(e.target.value) })
            }
          />
        </div>
        <button type="submit" disabled={updateDeal.isPending}>
          {updateDeal.isPending ? "Saving…" : "Save"}
        </button>
        {updateDeal.error && <p role="alert">{updateDeal.error.message}</p>}
      </form>
    </div>
  );
}
