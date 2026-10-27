import { Link } from "react-router";
import { useListContacts, useRemoveContact } from "../../api/contacts";

export function List() {
  const { data, isLoading, isError, error } = useListContacts();
  const removeContact = useRemoveContact();

  if (isLoading) {
    return <p>Loading…</p>;
  }

  if (isError) {
    return <p role="alert">{error.message}</p>;
  }

  return (
    <div>
      <h1>Contacts</h1>
      <p>
        <Link to="/contacts/new">New Contact</Link>
      </p>
      <table>
        <thead>
          <tr>
            <th>First name</th>
            <th>Last name</th>
            <th>Email</th>
            <th>Phone</th>
            <th>Title</th>
            <th>Ai summary</th>
            <th>Avatar key</th>
            <th>Company id</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {data?.items.map((contact) => (
            <tr key={contact.id}>
              <td>
                <Link to={`/contacts/${contact.id}`}>{contact.first_name}</Link>
              </td>
              <td>{contact.last_name}</td>
              <td>{contact.email}</td>
              <td>{contact.phone}</td>
              <td>{contact.title}</td>
              <td>{contact.ai_summary}</td>
              <td>{contact.avatar_key}</td>
              <td>{contact.company_id}</td>
              <td>
                <Link to={`/contacts/${contact.id}/edit`}>Edit</Link>{" "}
                <button
                  type="button"
                  disabled={removeContact.isPending}
                  onClick={() => removeContact.mutate(contact.id)}
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      {removeContact.isError && <p role="alert">{removeContact.error.message}</p>}
    </div>
  );
}
