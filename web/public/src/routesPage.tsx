import { createBrowserRouter, createRoutesFromElements, Route } from "react-router-dom";
import { Home } from "./Home";
import { Login } from "./Login";
import { TopPage } from "./TopPage";

export const routesPage = createBrowserRouter(
    createRoutesFromElements(
        <Route path="/" element={<TopPage />}>
            <Route path="/" element={<Home />} />
            <Route path="/home" element={<Home />} />
            <Route path="/login" element={<Login />} />
        </Route>
    )
)