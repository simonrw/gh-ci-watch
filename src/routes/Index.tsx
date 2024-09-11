import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { useEffect, useState } from "react";

type Pr = number;

export default function Index() {
  const [prs, setPrs] = useState([]);
  const [text, setText] = useState("");

  const addPr = async (e) => {
    e.preventDefault();
    const num = parseInt(text);
    if (Number.isNaN(num)) {
      // not a valid number
      setText("");
      return;
    }
    console.log(num);

    await invoke("add_pr", { prNumber: num });
    setText("");
  };

  const clearPrs = async (e) => {
    e.preventDefault();
    await invoke("clear_prs");
  };

  useEffect(() => {
    console.log("setting up listener");
    const unlisten = listen("state", (event) => {
      const { payload: prs } = event;
      console.log({ prs });
      setPrs(prs);
    });

    return () => {
      unlisten.then((f) => {
        console.log("removing listener");
        f();
      });
    };
  }, []);

  return (
    <div className="mx-auto bg-slate-950 p-2 flex flex-col">
      <h1 className="text-2xl text-center">Actions</h1>
      <input
        className="text-black"
        type="text"
        value={text}
        onChange={(e) => setText(e.target.value)}
      ></input>
      <button onClick={addPr}>Add PR</button>
      <button onClick={clearPrs}>Clear PRs</button>

      <div>
        <ul>
          {prs.map((pr) => {
            return <li key={pr}>{pr}</li>;
          })}
        </ul>
      </div>
    </div>
  );
}
