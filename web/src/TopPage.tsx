import { Link, Outlet } from "react-router-dom";

export const TopPage = () => <>
    <nav>
        <ul>    
            <li><Link to="/">TopPage</Link></li>
            <li><Link to="/home">Home</Link></li>
            <li><Link to="/login">Login</Link></li>
        </ul>
    </nav>
    <main>
        <Outlet />
    </main>
</>;