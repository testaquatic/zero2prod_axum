import { CookiesProvider } from "react-cookie";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "react-router-dom";
import { routesPage } from "./routesPage";


function App() {
  return (
  <>
    <QueryClientProvider client={new QueryClient()}>
    <CookiesProvider>
    <RouterProvider router={routesPage} />
    </CookiesProvider>
    </QueryClientProvider>
  </>
  );
}

export default App
