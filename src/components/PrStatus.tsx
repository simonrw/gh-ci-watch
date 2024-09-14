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
};

type PrStatusProps = {
  pr: Pr;
  removePr: (prNumber: number) => void;
};

export function PrStatus({ pr, removePr }: PrStatusProps) {
  const [prevStatus, setPrevStatus] = useState<Status | null>(null);
  const storage = useContext(StorageContext);

  const {
    data: status,
    isLoading,
    error,
  } = useQuery<StatusPayload>({
    queryKey: ["pr", pr.number],
    queryFn: async () => {
      const { status, title, description } = await invoke<PrStatusResponse>(
        "fetch_status",
        {
          owner: pr.owner,
          repo: pr.repo,
          prNumber: pr.number,
          token: storage.getToken(),
        }
      );
      return { status: statusFromRaw(status), title, description };
    },
    refetchInterval: 10000,
  });

  if (isLoading)
    return (
      <div>
        <p>
          {pr.owner}/{pr.repo} #{pr.number} ??
        </p>
      </div>
    );

  if (error)
    return (
      <div>
        <p>Error: {error.toString()}</p>
      </div>
    );

  console.log({ status });

  if (status!.status !== prevStatus) {
    setPrevStatus(status!.status);

    if (status!.status.kind === "succeeded") {
      new Notification("PR action succeeded!", {
        body: "The PR completed successfully",
      });
    } else if (status!.status.kind === "failed") {
      new Notification("PR action failed", {
        body: "The PR completed did not succeed",
      });
    }
  }

  let borderColor = "";
  switch (status!.status.kind) {
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
              <p>{status!.title}</p>
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
              <Markdown skipHtml>{status!.description}</Markdown>
            </CollapsibleContent>
          </Collapsible>
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ProgressReport status={status!.status} />
      </CardContent>
    </Card>
  );
}
