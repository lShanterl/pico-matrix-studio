import {useEffect, useState} from "react";
import {listen} from "@tauri-apps/api/event";
import {invoke} from "@tauri-apps/api/core";
import {ConnectionStatus} from "../types.ts";

export function usePicoConnection() {
    const [status, setStatus] = useState<ConnectionStatus>({ connected: false, ip: "" });

    useEffect(() => {
        const unlisten = listen<ConnectionStatus>("pico-connection-status", (e) => {
            setStatus(e.payload);
        });
        return () => { unlisten.then((fn) => fn()); };
    }, []);

    const connect = (ip: string) => invoke("connect_to_pico", { ip });
    const disconnect = () => invoke("disconnect_from_pico");

    return { status, connect, disconnect };
}