export type Pr = {
  status: Status;
  number: number;
  repo: string;
  workflowId: number;
  owner: string;
};

export type RawStatus =
  | "Queued"
  | { InProgress: number }
  | "Succeeded"
  | "Failed"
  | "Unknown";

export type QueuedStatus = {
  kind: "queued";
};

export type InProgressStatus = {
  kind: "in-progress";
  completion: number;
};

export type SucceededStatus = {
  kind: "succeeded";
};

export type FailedStatus = {
  kind: "failed";
};

export type UnknownStatus = {
  kind: "unknown";
};

export type Status =
  | QueuedStatus
  | InProgressStatus
  | SucceededStatus
  | FailedStatus
  | UnknownStatus;

export const statusFromRaw = (raw: RawStatus): Status => {
  switch (raw) {
    case "Queued":
      return { kind: "queued" };
    case "Succeeded":
      return { kind: "succeeded" };
    case "Failed":
      return { kind: "failed" };
    case "Unknown":
      return { kind: "unknown" };
    default:
      return { kind: "in-progress", completion: raw.InProgress };
  }
};

export type StatusPayload = {
  status: Status;
  title: string;
  number: number;
  description: string;
  numSteps: number;
  numCompleteSteps: number;
  prUrl: string;
  runUrl: string;
};
