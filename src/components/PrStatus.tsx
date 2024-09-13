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

type PrStatusResponse = {
  status: RawStatus;
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

  if (isError)
    return (
      <div>
        <p>Error</p>
      </div>
    );

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
