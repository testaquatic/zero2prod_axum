import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { checkCookieFeeder, getCookieFeeder } from "./cookiefeeder";


export const Login = () => {
    // https://www.npmjs.com/package/react-cookie 이 문서를 참고로 했다.
    const cookieFeeder = getCookieFeeder();
    const [message, setMessage] = useState<string|undefined>(undefined);
    const queryCheckCookieFeeder = useQuery(
        {queryKey: ["cookieFeeder"], queryFn: () => checkCookieFeeder({message: cookieFeeder.message, hmac: cookieFeeder.hmac}), enabled: false});
    useEffect(() => {
        queryCheckCookieFeeder.refetch().then((response) => {
            if (response.data === 200) {
                setMessage(cookieFeeder.message);
            } else {
                setMessage(undefined);
            }
        }).catch((error) => {
            setMessage(error);
        });
    }, [cookieFeeder.message]);
    
    return(<>
    {message && <div className="loginError">{decodeURI(message)}</div>}
    <form method="post" action="/login">
        <div className="inputUser">
            <div className="inputUsername">
                <label htmlFor="username">Username </label>
                <input type="text" name="username" id="username" placeholder="Enter Username" autoComplete="username" />
            </div>

            <div className="inputPassword">
                <label htmlFor="password">Password </label>
                <input type="password" name="password" id="password" placeholder="Enter Password" autoComplete="current-password"/>
            </div>

            <div className="buttonSubmit">
                <button type="submit">Login</button>
                <button type="reset">Reset</button>
            </div>
        </div>
    </form>
    </>);
};
