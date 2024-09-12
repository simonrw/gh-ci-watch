import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Pr } from "../types";
import { PrStatus } from "../components/PrStatus";
import { Button } from "@/components/ui/button";
import { ModeToggle } from "@/components/ui/ThemeModeToggle";
import { Input } from "@/components/ui/input";

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
    <div className="flex flex-col p-4 gap-8">
      <div className="flex items-center justify-between">
        <h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
          Actions
        </h1>
        <ModeToggle />
      </div>
      <div className="flex flex-col gap-4">
        <Input
          id="owner"
          placeholder="Owner"
          value={owner}
          onChange={(e) => setOwner(e.target.value)}
        ></Input>
        <Input
          id="repo"
          placeholder="Repo"
          value={repo}
          onChange={(e) => setRepo(e.target.value)}
        ></Input>
        <Input
          id="pr"
          placeholder="Pr number"
          type="text"
          value={text}
          onChange={(e) => {
            console.log({ value: e.target.value });
            if (!isNumeric(e.target.value)) {
              return;
            }
            setText(e.target.value);
          }}
        ></Input>
        <Button
          onClick={(e) => {
            e.preventDefault();
            addPr();
          }}
        >
          Track PR
        </Button>
      </div>

      <div className="flex flex-col gap-2">
        {prs.map((pr) => {
          return <PrStatus key={pr.number} pr={pr} />;
        })}
      </div>
    </div>
  );
}
