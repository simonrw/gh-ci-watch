export type Pr = {
  status: Status;
  number: number;
  repo: string;
  owner: string;
};

export type Status =
  | "Queued"
  | { InProgress: number }
  | "Succeeded"
  | "Failed"
  | "Unknown";
