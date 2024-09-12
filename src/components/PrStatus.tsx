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

export function PrStatus({ pr }: { pr: Pr }) {
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
          {pr.owner}/{pr.repo}
        </CardTitle>
        <CardDescription>#{pr.number}</CardDescription>
      </CardHeader>
      <CardContent>
        <ProgressReport status={status!} />
      </CardContent>
    </Card>
  );
}
