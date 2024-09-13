import { Pr } from "@/types";

const STORAGE_KEY = "store";

export class Storage {
  prs: Pr[];

  constructor(prs: Pr[] | undefined) {
    this.prs = prs || [];
  }

  public addPr(pr: Pr): void {
    this.prs.push(pr);
  }

  public save(): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.prs));
  }

  public static load(): Storage {
    let rawState = localStorage.getItem(STORAGE_KEY);
    let state;
    if (rawState) {
      state = JSON.parse(rawState);
    } else {
      state = [];
    }
    return new Storage(state);
  }
}
