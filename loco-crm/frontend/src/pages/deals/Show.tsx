import { Link, useParams } from "react-router";
import { useDeal } from "../../api/deals";

export function Show() {
  const { id: idParam } = useParams<{ id: string }>();
  const id = Number(idParam);
  const isValidId = Number.isFinite(id) && id > 0;

  const { data, isLoading, isError, error } = useDeal(isValidId ? id : NaN);

  if (!isValidId) {
    return <p role="alert">Invalid deal id.</p>;
  }

  if (isLoading) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  if (!data) {
    return <p>Deal not found.</p>;
  }

  return (
    <div>
      <h1>{data.title}</h1>
      <dl>
        <dt>Amount cents</dt>
        <dd>{data.amount_cents ?? "—"}</dd>
        <dt>Stage</dt>
        <dd>{data.stage}</dd>
        <dt>Company id</dt>
        <dd>{data.company_id ?? "—"}</dd>
        <dt>Contact id</dt>
        <dd>{data.contact_id ?? "—"}</dd>
        <dt>Created at</dt>
        <dd>{data.created_at}</dd>
        <dt>Updated at</dt>
        <dd>{data.updated_at}</dd>
      </dl>
      <p>
        <Link to={`/deals/${data.id}/edit`}>Edit</Link>{" "}
        <Link to="/deals">Back to list</Link>
      </p>
    </div>
  );
}
