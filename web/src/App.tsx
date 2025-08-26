import { RouterProvider } from 'react-router-dom';
import { routesPage } from './routesPage';
import { CookiesProvider } from 'react-cookie';

function App() {
  return <>
    <CookiesProvider>
    <RouterProvider router={routesPage} />
    </CookiesProvider>
  </>
}

export default App
