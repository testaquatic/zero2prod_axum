import { Link } from "react-router-dom";

export default function PasswordForm() {
    return (
        <>
        <form action="/admin/password" method="post">
            <div>
            <label htmlFor="current_password">Current password </label>
            <input type="password" id="current_password" name="current_password" placeholder="Enter current password." required />
            </div>
            <div>
                <label htmlFor="new_password">New password </label>
                <input type="password" id="new_password" name="new_password" placeholder="Enter new password." required />
            </div>
            <div>
                <label htmlFor="new_password_check">Confirm new password </label>
                <input type="password" id="new_password_check" name="new_password_check" placeholder="Type the new password again." required />
            </div>
            <div>
                <button type="submit">Change password</button>
            </div>
        </form>
        <div>
            <p>
                <Link to="/dashboard">&lt;-Back</Link>
            </p>
        </div>
        </>
    );
}