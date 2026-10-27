import { Link } from "react-router";
import { useListCompanies, useRemoveCompany } from "../../api/companies";

export function List() {
  const { data, isLoading, isError, error } = useListCompanies();
  const removeCompany = useRemoveCompany();

  if (isLoading) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  return (
    <div>
      <h1>Companies</h1>
      <p>
        <Link to="/companies/new">New Company</Link>
      </p>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Domain</th>
            <th>Industry</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {data?.items.map((company) => (
            <tr key={company.id}>
              <td>
                <Link to={`/companies/${company.id}`}>{company.name}</Link>
              </td>
              <td>{company.domain}</td>
              <td>{company.industry}</td>
              <td>
                <Link to={`/companies/${company.id}/edit`}>Edit</Link>{" "}
                <button
                  type="button"
                  disabled={removeCompany.isPending}
                  onClick={() => removeCompany.mutate(company.id)}
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {removeCompany.isError && <p role="alert">{removeCompany.error.message}</p>}
    </div>
  );
}
