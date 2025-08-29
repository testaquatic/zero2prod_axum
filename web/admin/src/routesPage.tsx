import { createBrowserRouter, createRoutesFromElements, Route } from "react-router-dom";
import { Dashboard } from "./Dashboard";

export const routesPage = createBrowserRouter(
    createRoutesFromElements(
        <Route path="/dashboard" element={<Dashboard />} />
    ), {basename: "/admin"}
)