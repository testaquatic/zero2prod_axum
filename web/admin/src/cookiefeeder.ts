import { useCookies } from "react-cookie";

export type CookieFeeder = {
    message?: string;
    username?: string;
    hmac?: string;
};
    
export async function checkCookieFeeder (data: CookieFeeder) {
    data.message = encodeURI(data.message??"");
    data.username = encodeURI(data.username??"");
    // 입력중에 하나가 비어 있으면 검증할 이유가 없다.
    if ((data.message === undefined && data.username === undefined) || data.hmac === undefined) {
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

export function getCookieFeeder(): CookieFeeder {
    const [cookieMessage] = useCookies<"_cookie_feeder_messsage", {_cookie_feeder_messsage?: string;}>(["_cookie_feeder_messsage"]);
    const [cookieUsername] = useCookies<"_cookie_feeder_username", {_cookie_feeder_username?: string;}>(["_cookie_feeder_username"]);
    const [cookieHmac] = useCookies<"_cookie_feeder_hmac", {_cookie_feeder_hmac?: string;}>(["_cookie_feeder_hmac"]);

    return ({
        message: cookieMessage._cookie_feeder_messsage,
        username: cookieUsername._cookie_feeder_username,
        hmac: cookieHmac._cookie_feeder_hmac
    });
}