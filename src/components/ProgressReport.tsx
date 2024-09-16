import { Status } from "@/types";
import { Progress } from "./ui/progress";

type ProgressReportProps = {
  status: Status;
  numSteps: number;
  numCompleteSteps: number;
};

export function ProgressReport({
  status,
  numSteps,
  numCompleteSteps,
}: ProgressReportProps) {
  let statusValue = 100;
  if (status.kind === "in-progress") {
    statusValue = status.completion * 100;
  } else if (status.kind === "queued") {
    statusValue = 0;
  }

  return (
    <div className="flex text-muted-foreground items-center gap-2">
      <Progress value={statusValue}></Progress>
      <span>
        {numSteps}/{numCompleteSteps}
      </span>
    </div>
  );
}
