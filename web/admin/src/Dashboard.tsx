import { useEffect, useState } from "react";

import { useQuery } from "@tanstack/react-query";
import { checkCookieFeeder, getCookieFeeder } from "./cookiefeeder";

export function Dashboard() {
    const cookieFeeder = getCookieFeeder();
    const [userName, setUserName] = useState<string | undefined>(undefined);
    const queryCheckCookieFeeder  = useQuery(
        {queryKey: ["cookieFeeder"], queryFn: () => checkCookieFeeder({username: cookieFeeder.username, hmac: cookieFeeder.hmac}), enabled: false }, 
    );
    useEffect(() => {
        queryCheckCookieFeeder.refetch().then(response => {
            if (response.data === 200) {
                setUserName(cookieFeeder.username);
            } else {
                setUserName(undefined);
            }
        }).catch(error => setUserName(error))
    }, [cookieFeeder.username])
    return (
        <>
            <p>
                Welcome! {userName}
            </p>
        </>
    );
}
// everythinghastostartsomewhere