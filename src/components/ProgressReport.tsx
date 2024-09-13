import { Status } from "@/types";
import { Progress } from "./ui/progress";

type ProgressReportProps = {
  status: Status;
};

export function ProgressReport({ status }: ProgressReportProps) {
  let statusValue = 100;
  if (status.kind === "in-progress") {
    statusValue = status.completion * 100;
  }
  console.log({ statusValue });
  return <Progress value={statusValue}></Progress>;
}
