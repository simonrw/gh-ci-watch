import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { Pr } from "../types";
import { PrStatus } from "../components/PrStatus";
import { Button } from "@/components/ui/button";
import { ModeToggle } from "@/components/ui/ThemeModeToggle";
import { Input } from "@/components/ui/input";
import { InputForm } from "@/components/InputForm";

function isNumeric(str: string) {
  if (typeof str != "string") return false; // we only process strings!
  return (
    !isNaN(str) && // use type coercion to parse the _entirety_ of the string (`parseFloat` alone does not do this)...
    !isNaN(parseFloat(str))
  ); // ...and ensure strings of whitespace fail
}

export default function Index() {
  const [prs, setPrs] = useState<Pr[]>([]);

  const addPr = (pr: Pr) => {
    setPrs((prs) => [...prs, pr]);
  };

  return (
    <div className="flex flex-col p-4 gap-8">
      <div className="flex items-center justify-between">
        <h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
          Actions
        </h1>
        <ModeToggle />
      </div>
      <InputForm addPr={addPr} />
      <div className="flex flex-col gap-2">
        {prs.map((pr) => {
          return <PrStatus key={pr.number} pr={pr} />;
        })}
      </div>
    </div>
  );
}
