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
import {
  ChevronsUpDown,
  ClipboardCheck,
  GitPullRequestArrow,
} from "lucide-react";
import { StorageContext } from "@/lib/storage";

type PrStatusResponse = {
  status: RawStatus;
  title: string;
  description: string;
  num_steps: number;
  num_complete_steps: number;
  pr_url: string;
  run_url: string;
};

type PrStatusProps = {
  pr: Pr;
  removePr: (prNumber: number) => void;
};

export function PrStatus({ pr, removePr }: PrStatusProps) {
  const [prevStatus, setPrevStatus] = useState<Status | null>(null);
  const storage = useContext(StorageContext);

  const { data, isLoading, error } = useQuery<StatusPayload>({
    queryKey: ["pr", pr.number],
    queryFn: async () => {
      const response = await invoke<PrStatusResponse>("fetch_status", {
        owner: pr.owner,
        repo: pr.repo,
        prNumber: pr.number,
        workflowId: pr.workflowId,
        token: storage.getToken(),
      });

      return {
        status: statusFromRaw(response.status),
        title: response.title,
        description: response.description,
        numSteps: response.num_steps,
        numCompleteSteps: response.num_complete_steps,
        prUrl: response.pr_url,
        runUrl: response.run_url,
      };
    },
    refetchInterval: 10000,
  });

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
            <p className="flex gap-4 items-center">
              <span className="flex gap-2">
                <span className="text-xs underline text-muted-foreground">
                  <a href={data.prUrl} target="_blank">
                    <GitPullRequestArrow />
                  </a>
                </span>
                <span className="text-xs underline text-muted-foreground">
                  <a href={data.runUrl} target="_blank">
                    <ClipboardCheck />
                  </a>
                </span>
              </span>
              <span>{data.title}</span>
            </p>
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
