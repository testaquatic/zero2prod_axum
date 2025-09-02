import { CookiesProvider } from "react-cookie";
import { Dashboard } from "./Dashboard";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";


function App() {
  return (
  <>
    <QueryClientProvider client={new QueryClient()}>
    <CookiesProvider>
    <Dashboard />
    </CookiesProvider>
    </QueryClientProvider>
  </>
  );
}

export default App
