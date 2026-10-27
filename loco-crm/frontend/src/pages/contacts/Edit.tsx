import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { useContact, useUpdateContact } from "../../api/contacts";
import type { UpdateContact } from "../../bindings/UpdateContact";

export function Edit() {
  const navigate = useNavigate();
  const { id: idParam } = useParams<{ id: string }>();
  const id = Number(idParam);
  const isValidId = Number.isFinite(id) && id > 0;

  const { data, isLoading, isError, error } = useContact(isValidId ? id : NaN);
  const updateContact = useUpdateContact();

  const [form, setForm] = useState<UpdateContact | null>(null);

  useEffect(() => {
    if (data) {
      setForm({
        first_name: data.first_name,
        last_name: data.last_name,
        email: data.email,
        phone: data.phone,
        title: data.title,
        ai_summary: data.ai_summary,
        avatar_key: data.avatar_key,
        company_id: data.company_id,
      });
    }
  }, [data]);

  if (!isValidId) {
    return <p role="alert">Invalid contact id.</p>;
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
    updateContact.mutate(
      { id, data: form },
      {
        onSuccess: () => {
          navigate(`/contacts/${id}`);
        },
      },
    );
  }

  return (
    <div>
      <h1>Edit Contact</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="first_name">First name</label>
          <input
            id="first_name"
            type="text"
            value={form.first_name}
            onChange={(e) => setForm({ ...form, first_name: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="last_name">Last name</label>
          <input
            id="last_name"
            type="text"
            value={form.last_name}
            onChange={(e) => setForm({ ...form, last_name: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="email">Email</label>
          <input
            id="email"
            type="text"
            value={form.email}
            onChange={(e) => setForm({ ...form, email: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="phone">Phone</label>
          <input
            id="phone"
            type="text"
            value={form.phone}
            onChange={(e) => setForm({ ...form, phone: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="title">Title</label>
          <input
            id="title"
            type="text"
            value={form.title}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="ai_summary">Ai summary</label>
          <textarea
            id="ai_summary"
            value={form.ai_summary}
            onChange={(e) => setForm({ ...form, ai_summary: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="avatar_key">Avatar key</label>
          <input
            id="avatar_key"
            type="text"
            value={form.avatar_key}
            onChange={(e) => setForm({ ...form, avatar_key: e.target.value })}
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
        <button type="submit" disabled={updateContact.isPending}>
          {updateContact.isPending ? "Saving…" : "Save"}
        </button>
        {updateContact.error && <p role="alert">{updateContact.error.message}</p>}
      </form>
    </div>
  );
}
