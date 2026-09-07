import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

//const SIZE = 16;
//const _PIXEL_COUNT = SIZE * SIZE;

type RGB = [number, number, number];

export default function App() {
    const [input, setInput] = useState<string>('');
    const [_value, _setValue] = useState<RGB>();

    return (
        <main className="App">
            <input type={"text"} value={input} onChange={(e) => setInput(e.target.value)} />
            <button onClick={() => invoke("ping",{str:input})}>ping</button>
        </main>

  );
}

