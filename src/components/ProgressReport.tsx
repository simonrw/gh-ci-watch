import { Status } from "@/types";
import { Progress } from "./ui/progress";

type ProgressReportProps = {
  status: Status;
};

export function ProgressReport({ status }: ProgressReportProps) {
  let statusValue = 100;
  if (status.kind === "in-progress") {
    statusValue = status.completion * 100;
  } else if (status.kind === "queued") {
    statusValue = 0;
  }
  return <Progress value={statusValue}></Progress>;
}
