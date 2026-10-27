import { Link } from "react-router";
import { useListDeals, useRemoveDeal } from "../../api/deals";

export function List() {
  const { data, isLoading, isError, error } = useListDeals();
  const removeDeal = useRemoveDeal();

  if (isLoading) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  return (
    <div>
      <h1>Deals</h1>
      <p>
        <Link to="/deals/new">New Deal</Link>
      </p>
      <table>
        <thead>
          <tr>
            <th>Title</th>
            <th>Amount cents</th>
            <th>Stage</th>
            <th>Company id</th>
            <th>Contact id</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {data?.items.map((deal) => (
            <tr key={deal.id}>
              <td>
                <Link to={`/deals/${deal.id}`}>{deal.title}</Link>
              </td>
              <td>{deal.amount_cents}</td>
              <td>{deal.stage}</td>
              <td>{deal.company_id}</td>
              <td>{deal.contact_id}</td>
              <td>
                <Link to={`/deals/${deal.id}/edit`}>Edit</Link>{" "}
                <button
                  type="button"
                  disabled={removeDeal.isPending}
                  onClick={() => removeDeal.mutate(deal.id)}
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {removeDeal.isError && <p role="alert">{removeDeal.error.message}</p>}
    </div>
  );
}
