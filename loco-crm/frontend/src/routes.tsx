import { createBrowserRouter } from 'react-router'
import { App } from './App'
import { RequireAuth } from './auth/RequireAuth'
import { Home } from './pages/Home'
// scaffold:imports
import { Edit as DealsEdit } from './pages/deals/Edit'
import { List as DealsList } from './pages/deals/List'
import { New as DealsNew } from './pages/deals/New'
import { Show as DealsShow } from './pages/deals/Show'
import { Edit as ContactsEdit } from './pages/contacts/Edit'
import { List as ContactsList } from './pages/contacts/List'
import { New as ContactsNew } from './pages/contacts/New'
import { Show as ContactsShow } from './pages/contacts/Show'
import { Edit as CompaniesEdit } from './pages/companies/Edit'
import { List as CompaniesList } from './pages/companies/List'
import { New as CompaniesNew } from './pages/companies/New'
import { Show as CompaniesShow } from './pages/companies/Show'

// Mounted under /app; login/register live on the server-rendered side (/login).
export const router = createBrowserRouter([
  {
    path: '/',
    element: <App />,
    children: [
      { index: true, element: <Home /> },
      {
        element: <RequireAuth />,
        children: [
          // scaffold:routes
          { path: 'deals', element: <DealsList /> },
          { path: 'deals/new', element: <DealsNew /> },
          { path: 'deals/:id', element: <DealsShow /> },
          { path: 'deals/:id/edit', element: <DealsEdit /> },
          { path: 'contacts', element: <ContactsList /> },
          { path: 'contacts/new', element: <ContactsNew /> },
          { path: 'contacts/:id', element: <ContactsShow /> },
          { path: 'contacts/:id/edit', element: <ContactsEdit /> },
          { path: 'companies', element: <CompaniesList /> },
          { path: 'companies/new', element: <CompaniesNew /> },
          { path: 'companies/:id', element: <CompaniesShow /> },
          { path: 'companies/:id/edit', element: <CompaniesEdit /> },
        ],
      },
    ],
  },
], { basename: '/app' })
