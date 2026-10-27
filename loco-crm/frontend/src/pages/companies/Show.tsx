import { Link, useParams } from "react-router";
import { useCompany } from "../../api/companies";

export function Show() {
  const { id: idParam } = useParams<{ id: string }>();
  const id = Number(idParam);
  const isValidId = Number.isFinite(id) && id > 0;

  const { data, isLoading, isError, error } = useCompany(isValidId ? id : NaN);

  if (!isValidId) {
    return <p role="alert">Invalid company id.</p>;
  }

  if (isLoading) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  if (!data) {
    return <p>Company not found.</p>;
  }

  return (
    <div>
      <h1>{data.name}</h1>
      <dl>
        <dt>Domain</dt>
        <dd>{data.domain ?? "—"}</dd>
        <dt>Industry</dt>
        <dd>{data.industry ?? "—"}</dd>
        <dt>Created at</dt>
        <dd>{data.created_at}</dd>
        <dt>Updated at</dt>
        <dd>{data.updated_at}</dd>
      </dl>
      <p>
        <Link to={`/companies/${data.id}/edit`}>Edit</Link>{" "}
        <Link to="/companies">Back to list</Link>
      </p>
    </div>
  );
}
