import { Pr } from "@/types";
import { createContext } from "react";

const STORAGE_KEY = "store";

type State = {
  prs: Pr[];
  token: string | null;
};

export class Storage {
  state: State;

  constructor(state: State | undefined) {
    this.state = state || { prs: [], token: null };
  }

  public addPr(pr: Pr): void {
    this.state.prs.push(pr);
    this.save();
  }

  public removePr(prNumber: number): void {
    this.state.prs = this.state.prs.filter((pr) => pr.number !== prNumber);
    this.save();
  }

  public getToken(): string | null {
    return this.state.token;
  }

  public setToken(token: string) {
    this.state.token = token;
    this.save();
  }

  public save(): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.state));
  }

  public static load(): Storage {
    let rawState = localStorage.getItem(STORAGE_KEY);
    let state;
    if (rawState) {
      state = JSON.parse(rawState);
    } else {
      state = { prs: [], token: null };
    }
    return new Storage(state);
  }
}

export const StorageContext = createContext(Storage.load());
