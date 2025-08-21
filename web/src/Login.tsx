export const Login = () => <>
    <form method="post" action="/login">
        <div className="inputUser">
            <div className="inputUsername">
                <label htmlFor="username">Username </label>
                <input type="text" name="username" id="username" placeholder="Enter Username" />
            </div>

            <div className="inputPassword">
                <label htmlFor="password">Password </label>
                <input type="password" name="password" id="password" placeholder="Enter Password" />
            </div>

            <div className="buttonSubmit">
                <button type="submit">Login</button>
            </div>
        </div>
    </form>
</>