import { Link, useParams } from "react-router";
import { useContact } from "../../api/contacts";

export function Show() {
  const { id: idParam } = useParams<{ id: string }>();
  const id = Number(idParam);
  const isValidId = Number.isFinite(id) && id > 0;

  const { data, isLoading, isError, error } = useContact(isValidId ? id : NaN);

  if (!isValidId) {
    return <p role="alert">Invalid contact id.</p>;
  }

  if (isLoading) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  if (!data) {
    return <p>Contact not found.</p>;
  }

  return (
    <div>
      <h1>{data.first_name}</h1>
      <dl>
        <dt>Last name</dt>
        <dd>{data.last_name}</dd>
        <dt>Email</dt>
        <dd>{data.email}</dd>
        <dt>Phone</dt>
        <dd>{data.phone ?? "—"}</dd>
        <dt>Title</dt>
        <dd>{data.title ?? "—"}</dd>
        <dt>Ai summary</dt>
        <dd>{data.ai_summary ?? "—"}</dd>
        <dt>Avatar key</dt>
        <dd>{data.avatar_key ?? "—"}</dd>
        <dt>Company id</dt>
        <dd>{data.company_id ?? "—"}</dd>
        <dt>Created at</dt>
        <dd>{data.created_at}</dd>
        <dt>Updated at</dt>
        <dd>{data.updated_at}</dd>
      </dl>
      <p>
        <Link to={`/contacts/${data.id}/edit`}>Edit</Link>{" "}
        <Link to="/contacts">Back to list</Link>
      </p>
    </div>
  );
}
