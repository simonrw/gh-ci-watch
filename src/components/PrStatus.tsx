import { useQuery } from "@tanstack/react-query";
import { Pr, Status } from "../types";
import { invoke } from "@tauri-apps/api/tauri";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { ProgressReport } from "./ProgressReport";
import { Button } from "./ui/button";
import { Trash } from "lucide-react";

type PrStatusResponse = {
  status: Status;
};

const statusToString = (status: Status): string => {
  switch (status) {
    case "Queued":
      return "Queued";
    case "Succeeded":
      return "Succeeded";
    case "Failed":
      return "Failed";
    case "Unknown":
      return "Unknown";
    default:
      if (status && status["InProgress"]) {
        return `InProgress ${status.InProgress}%`;
      }

      return "??";
  }
};

type PrStatusProps = {
  pr: Pr;
  removePr: (prNumber: number) => void;
};

export function PrStatus({ pr, removePr }: PrStatusProps) {
  const {
    data: status,
    isLoading,
    isError,
  } = useQuery<Status>({
    queryKey: ["pr", pr.number],
    queryFn: async () => {
      const { status } = await invoke<PrStatusResponse>("fetch_status", {
        owner: pr.owner,
        repo: pr.repo,
        prNumber: pr.number,
      });
      return status;
    },
    refetchInterval: 5000,
  });

  if (isLoading)
    return (
      <div>
        <p>
          {pr.owner}/{pr.repo} #{pr.number} ??
        </p>
      </div>
    );

  if (isError)
    return (
      <div>
        <p>Error</p>
      </div>
    );

  let borderColor = "";
  switch (status!) {
    case "Succeeded":
      borderColor = "border border-green-500";
      break;
    case "Failed":
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
              <Button variant="destructive" onClick={() => removePr(pr.number)}>
                <Trash className="h-4 w-4" />
              </Button>
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
