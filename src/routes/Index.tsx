import { useContext, useState } from "react";
import { Pr } from "../types";
import { PrStatus } from "../components/PrStatus";
import { StorageContext } from "@/lib/storage";
import { Navigate } from "react-router-dom";
import { Header } from "@/components/Header";

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
      <Header addPr={addPr} />
      <div className="flex flex-col gap-2">
        {prs.map((pr) => {
          return <PrStatus key={pr.number} pr={pr} removePr={removePr} />;
        })}
      </div>
    </div>
  );
}
