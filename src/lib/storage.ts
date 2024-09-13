import { Pr } from "@/types";
import { createContext } from "react";

const STORAGE_KEY = "store";

export class Storage {
  prs: Pr[];

  constructor(prs: Pr[] | undefined) {
    this.prs = prs || [];
  }

  public addPr(pr: Pr): void {
    this.prs.push(pr);
    this.save();
  }

  public removePr(prNumber: number): void {
    this.prs = this.prs.filter(pr => pr.number !== prNumber);
    this.save();
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

export const StorageContext = createContext(Storage.load());
