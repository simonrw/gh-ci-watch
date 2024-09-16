import { useQuery } from "@tanstack/react-query";
import { Pr, RawStatus, Status, statusFromRaw, StatusPayload } from "../types";
import { invoke } from "@tauri-apps/api/tauri";
import Markdown from "react-markdown";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { ProgressReport } from "./ProgressReport";
import { DeleteButton } from "./DeleteButton";
import { useContext, useState } from "react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "./ui/collapsible";
import { ChevronsUpDown } from "lucide-react";
import { StorageContext } from "@/lib/storage";

type PrStatusResponse = {
  status: RawStatus;
  title: string;
  description: string;
  num_steps: number;
  num_complete_steps: number;
};

type PrStatusProps = {
  pr: Pr;
  removePr: (prNumber: number) => void;
};

export function PrStatus({ pr, removePr }: PrStatusProps) {
  console.log({ pr });
  const [prevStatus, setPrevStatus] = useState<Status | null>(null);
  const storage = useContext(StorageContext);

  const { data, isLoading, error } = useQuery<StatusPayload>({
    queryKey: ["pr", pr.number],
    queryFn: async () => {
      const { status, title, description, num_steps, num_complete_steps } =
        await invoke<PrStatusResponse>("fetch_status", {
          owner: pr.owner,
          repo: pr.repo,
          prNumber: pr.number,
          token: storage.getToken(),
        });
      return {
        status: statusFromRaw(status),
        title,
        description,
        numSteps: num_steps,
        numCompleteSteps: num_complete_steps,
      };
    },
    refetchInterval: 10000,
  });
  console.log({ isLoading, data, error });

  if (error)
    return (
      <Card className="border border-red-300">
        <CardHeader>
          <CardTitle>
            <div className="flex justify-between">
              <div>
                <p>Error</p>
              </div>
              <DeleteButton pr={pr.number} removePr={removePr} />
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent>{error.toString()}</CardContent>
      </Card>
    );

  if (isLoading || !data) {
    return (
      <Card className="border border-yellow-600 animate-pulse">
        <CardHeader>
          <CardTitle>
            <div className="flex justify-between">
              <div>
                <p>Loading...</p>
              </div>
              <DeleteButton pr={pr.number} removePr={removePr} />
            </div>
          </CardTitle>
        </CardHeader>
        <CardContent></CardContent>
      </Card>
    );
  }

  if (data.status !== prevStatus) {
    setPrevStatus(data.status);

    if (data.status.kind === "succeeded") {
      new Notification("PR action succeeded!", {
        body: "The PR completed successfully",
      });
    } else if (data.status.kind === "failed") {
      new Notification("PR action failed", {
        body: "The PR completed did not succeed",
      });
    }
  }

  let borderColor = "";
  switch (data.status.kind) {
    case "succeeded":
      borderColor = "border border-green-500";
      break;
    case "in-progress":
      borderColor = "animate-pulse";
      break;
    case "queued":
      borderColor = "animate-pulse";
      break;
    case "failed":
      borderColor = "border border-red-500";
      break;
  }

  return (
    <Card className={borderColor}>
      <CardHeader>
        <CardTitle>
          <div className="flex justify-between">
            <div>
              <p>{data.title}</p>
            </div>
            <DeleteButton pr={pr.number} removePr={removePr} />
          </div>
        </CardTitle>
        <CardDescription>
          <Collapsible>
            <CollapsibleTrigger className="flex items-center gap-2">
              <p>
                {pr.owner}/{pr.repo} (#{pr.number})
              </p>
              <ChevronsUpDown />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <Markdown skipHtml>{data.description}</Markdown>
            </CollapsibleContent>
          </Collapsible>
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ProgressReport
          status={data.status}
          numCompleteSteps={data.numCompleteSteps}
          numSteps={data.numSteps}
        />
      </CardContent>
    </Card>
  );
}
