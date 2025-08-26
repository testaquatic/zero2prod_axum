import { useCookies } from "react-cookie";

export const Login = () => {
    // https://www.npmjs.com/package/react-cookie 이 문서를 참고로 했다.
    const [flashCookie, _] = useCookies<"_flash", {_flash?: string;}>(["_flash"]);


    return<>
    <div className="loginError">{flashCookie._flash}</div>
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
};