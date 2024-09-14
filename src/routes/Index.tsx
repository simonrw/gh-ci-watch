import { useContext, useState } from "react";
import { Pr } from "../types";
import { PrStatus } from "../components/PrStatus";
import { Button } from "@/components/ui/button";
import { ModeToggle } from "@/components/ui/ThemeModeToggle";
import { InputForm } from "@/components/InputForm";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { StorageContext } from "@/lib/storage";
import { Navigate } from "react-router-dom";

export default function Index() {
  const storage = useContext(StorageContext);

  if (!storage.state.token) {
    return <Navigate replace to="/auth" />;
  }

  const [prs, setPrs] = useState<Pr[]>(storage.state.prs);

  const addPr = (pr: Pr) => {
    setPrs((prs) => [...prs, pr]);
    storage.addPr(pr);
  };

  const removePr = (prNumber: number) => {
    setPrs((prs) => prs.filter((pr) => pr.number !== prNumber));
    storage.removePr(prNumber);
  };

  return (
    <div className="flex flex-col p-4 gap-8">
      <div className="flex items-center justify-between">
        <h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
          Actions
        </h1>
        <div className="flex items-center gap-4">
          <Popover>
            <PopoverTrigger asChild>
              <Button>Add PR</Button>
            </PopoverTrigger>
            <PopoverContent>
              <InputForm addPr={addPr} />
            </PopoverContent>
          </Popover>
          <ModeToggle />
        </div>
      </div>
      <div className="flex flex-col gap-2">
        {prs.map((pr) => {
          return <PrStatus key={pr.number} pr={pr} removePr={removePr} />;
        })}
      </div>
    </div>
  );
}
