import { useState } from "react";
import { useNavigate } from "react-router";
import { useCreateContact } from "../../api/contacts";
import type { CreateContact } from "../../bindings/CreateContact";

export function New() {
  const navigate = useNavigate();
  const createContact = useCreateContact();

  const [form, setForm] = useState<CreateContact>({
    first_name: "",
    last_name: "",
    email: "",
    phone: null,
    title: null,
    ai_summary: null,
    avatar_key: null,
    company_id: null,
  });

  function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    createContact.mutate(form, {
      onSuccess: (created) => {
        navigate(`/contacts/${created.id}`);
      },
    });
  }

  return (
    <div>
      <h1>New Contact</h1>
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="first_name">First name</label>
          <input
            id="first_name"
            type="text"
            value={form.first_name ?? ""}
            onChange={(e) => setForm({ ...form, first_name: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="last_name">Last name</label>
          <input
            id="last_name"
            type="text"
            value={form.last_name ?? ""}
            onChange={(e) => setForm({ ...form, last_name: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="email">Email</label>
          <input
            id="email"
            type="text"
            value={form.email ?? ""}
            onChange={(e) => setForm({ ...form, email: e.target.value })}
            required
          />
        </div>
        <div>
          <label htmlFor="phone">Phone</label>
          <input
            id="phone"
            type="text"
            value={form.phone ?? ""}
            onChange={(e) => setForm({ ...form, phone: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="title">Title</label>
          <input
            id="title"
            type="text"
            value={form.title ?? ""}
            onChange={(e) => setForm({ ...form, title: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="ai_summary">Ai summary</label>
          <textarea
            id="ai_summary"
            value={form.ai_summary ?? ""}
            onChange={(e) => setForm({ ...form, ai_summary: e.target.value })}
          />
        </div>
        <div>
          <label htmlFor="avatar_key">Avatar key</label>
          <input
            id="avatar_key"
            type="text"
            value={form.avatar_key ?? ""}
            onChange={(e) => setForm({ ...form, avatar_key: e.target.value })}
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
        <button type="submit" disabled={createContact.isPending}>
          {createContact.isPending ? "Creating…" : "Create"}
        </button>
        {createContact.error && <p role="alert">{createContact.error.message}</p>}
      </form>
    </div>
  );
}
