import { Status } from "@/types";
import { Progress } from "./ui/progress";

type ProgressReportProps = {
  status: Status;
};

export function ProgressReport({ status }: ProgressReportProps) {
  if (status && status["InProgress"]) {
    return <p>In progress</p>;
  } else {
    return <Progress value={100}></Progress>;
  }
}
