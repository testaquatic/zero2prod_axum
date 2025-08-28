import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useCookies } from "react-cookie";

const checkFlash = async (data: {message: string|undefined, hmac: string|undefined}) => {
    // 입력중에 하나가 비어 있으면 검증할 이유가 없다.
    if (data.message === undefined || data.hmac === undefined) {
        return;
    }

    const body = JSON.stringify(data);
    // https://developer.mozilla.org/ko/docs/Web/API/Fetch_API/Using_Fetch 이 문서를 참고로 했다.
    const response = await fetch("/check/hmac", {
        method: "POST",
        headers: {
            "Content-Type": "application/json"
        },
        body
    });

    return response.status;
}

export const Login = () => {
    // https://www.npmjs.com/package/react-cookie 이 문서를 참고로 했다.
    const [flashCookie] = useCookies<"_flash", {_flash?: string;}>(["_flash"]);
    const [flashHmacCookie] = useCookies<"_flash_hmac", {_flash_hmac?: string;}>(["_flash_hmac"]);
    const [message, setMessage] = useState<string|undefined>(undefined);
    const queryCheckFlash = useQuery({queryKey: ["checkFlash"], queryFn: () => checkFlash({message: flashCookie._flash, hmac: flashHmacCookie._flash_hmac}), enabled: false});
    useEffect(() => {
        queryCheckFlash.refetch().then((response) => {
            if (response.data === 200) {
                setMessage(flashCookie._flash);
            } else {
                setMessage(undefined);
            }
        }).catch((error) => {
            setMessage(error);
        });
    }, [flashCookie, flashHmacCookie]);
    
    return(<>
    {message && <div className="loginError">{message}</div>}
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