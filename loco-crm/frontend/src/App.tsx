import { Link, Outlet } from 'react-router'

export function App() {
  return (
    <div className="container">
      <nav className="row" style={{ marginBottom: 16 }}>
        <Link to="/">React app</Link>
        <Link to="/companies">Companies</Link>
        <Link to="/contacts">Contacts</Link>
        <Link to="/deals">Deals</Link>
        <a href="/dashboard">← Back to server-rendered CRM</a>
        <form method="post" action="/logout" style={{ margin: 0 }}>
          <button type="submit" className="link">Log out</button>
        </form>
      </nav>
      <Outlet />
    </div>
  )
}
