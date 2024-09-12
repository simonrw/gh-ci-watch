import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Pr } from "../types";
import { PrStatus } from "../components/PrStatus";

function isNumeric(str: string) {
  if (typeof str != "string") return false; // we only process strings!
  return (
    !isNaN(str) && // use type coercion to parse the _entirety_ of the string (`parseFloat` alone does not do this)...
    !isNaN(parseFloat(str))
  ); // ...and ensure strings of whitespace fail
}

export default function Index() {
  const [prs, setPrs] = useState<Pr[]>([]);
  const [owner, setOwner] = useState("");
  const [repo, setRepo] = useState("");
  const [text, setText] = useState("");

  useEffect(() => {
    console.log("setting up listener");
    const unlisten = listen("state", (event) => {
      const { payload: prs } = event as { payload: Pr[] };
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

  const addPr = () => {
    // validation
    if (!owner) {
      console.error("no owner");
      return;
    }

    if (!repo) {
      console.error("no repo");
      return;
    }

    if (!text) {
      console.error("no text");
      return;
    }

    const newPr: Pr = {
      owner,
      repo,
      number: parseInt(text),
      status: "Unknown",
    };
    console.log("Adding pr");
    setPrs((prs) => [...prs, newPr]);
  };

  return (
    <div className="mx-auto bg-slate-950 p-2 flex flex-col">
      <h1 className="text-2xl text-center">Actions</h1>
      <div className="flex flex-col">
        <label htmlFor="owner">Owner</label>
        <input
          id="owner"
          className="text-black"
          value={owner}
          onChange={(e) => setOwner(e.target.value)}
        ></input>
        <label htmlFor="repo">Repo</label>
        <input
          id="repo"
          className="text-black"
          value={repo}
          onChange={(e) => setRepo(e.target.value)}
        ></input>
        <label htmlFor="pr">Pr #</label>
        <input
          id="pr"
          className="text-black"
          type="text"
          value={text}
          onChange={(e) => {
            console.log({ value: e.target.value });
            if (!isNumeric(e.target.value)) {
              return;
            }
            setText(e.target.value);
          }}
        ></input>
        <button
          onClick={(e) => {
            e.preventDefault();
            addPr();
          }}
        >
          Track PR
        </button>
      </div>

      <div>
        <ul>
          {prs.map((pr) => {
            return <PrStatus key={pr.number} pr={pr} />;
          })}
        </ul>
      </div>
    </div>
  );
}
