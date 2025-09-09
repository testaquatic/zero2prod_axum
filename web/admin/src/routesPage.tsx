import { createBrowserRouter, createRoutesFromElements, Route } from "react-router-dom";
import { Dashboard } from "./Dashboard";
import PasswordForm from "./Password";

export const routesPage = createBrowserRouter(
    createRoutesFromElements(
        <Route path="/" >
            <Route path="/dashboard" element={<Dashboard />} />
            <Route path="/password" element={<PasswordForm />} />
        </Route>
    ), {basename: "/admin"}
)