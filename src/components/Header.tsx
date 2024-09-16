import { Pr } from "@/types";
import { InputForm } from "./InputForm";
import { Button } from "./ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { ModeToggle } from "./ui/ThemeModeToggle";

type HeaderProps = {
  addPr: (pr: Pr) => void;
};

export function Header({ addPr }: HeaderProps) {
  return (
    <div className="flex items-center justify-between">
      <h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
        Actions
      </h1>
      <div className="flex items-center gap-4">
        <Popover>
          <PopoverTrigger asChild>
            <Button>Add PR</Button>
          </PopoverTrigger>
          <PopoverContent>
            <InputForm addPr={addPr} />
          </PopoverContent>
        </Popover>
        <ModeToggle />
      </div>
    </div>
  );
}
