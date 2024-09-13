import { useQuery } from "@tanstack/react-query";
import { Pr, RawStatus, Status, statusFromRaw } from "../types";
import { invoke } from "@tauri-apps/api/tauri";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { ProgressReport } from "./ProgressReport";
import { DeleteButton } from "./DeleteButton";
import { useState } from "react";

type PrStatusResponse = {
  status: RawStatus;
};

type PrStatusProps = {
  pr: Pr;
  removePr: (prNumber: number) => void;
};

export function PrStatus({ pr, removePr }: PrStatusProps) {
  const [prevStatus, setPrevStatus] = useState<Status | null>(null);

  const {
    data: status,
    isLoading,
    error,
  } = useQuery<Status>({
    queryKey: ["pr", pr.number],
    queryFn: async () => {
      const { status } = await invoke<PrStatusResponse>("fetch_status", {
        owner: pr.owner,
        repo: pr.repo,
        prNumber: pr.number,
      });
      return statusFromRaw(status);
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

  if (status! !== prevStatus) {
    setPrevStatus(status!);

    if (status!.kind === "succeeded") {
      new Notification("PR action succeeded!", {
        body: "The PR completed successfully",
      });
    } else if (status!.kind === "failed") {
      new Notification("PR action failed", {
        body: "The PR completed did not succeed",
      });
    }
  }

  let borderColor = "";
  switch (status!.kind) {
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
            <span>
              {pr.owner}/{pr.repo}
            </span>
            <span>
              <DeleteButton pr={pr.number} removePr={removePr} />
            </span>
          </div>
        </CardTitle>
        <CardDescription>#{pr.number}</CardDescription>
      </CardHeader>
      <CardContent>
        <ProgressReport status={status!} />
      </CardContent>
    </Card>
  );
}
