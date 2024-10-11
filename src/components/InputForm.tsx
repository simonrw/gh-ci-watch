import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "./ui/form";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Pr } from "@/types";
import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useContext } from "react";
import { StorageContext } from "@/lib/storage";
import {
  Select,
  SelectItem,
  SelectContent,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

const formSchema = z.object({
  owner: z.string().min(1).max(50, {
    message: "Owner must be less than 50 characters",
  }),
  repo: z.string().min(1).max(50, {
    message: "Repo must be less than 50 characters",
  }),
  workflow: z.coerce.number().min(0),
  pr: z.coerce.number().min(0),
});

type InputFormProps = {
  addPr: (pr: Pr) => void;
};

type Workflow = {
  id: number;
  name: string;
  filename: string;
};

export function InputForm(props: InputFormProps) {
  const storage = useContext(StorageContext);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      owner: "",
      repo: "",
      workflow: 0,
      pr: 0,
    },
  });

  const w = form.watch();
  const { data: workflows } = useQuery<Workflow[]>({
    queryKey: ["workflows", w.owner, w.repo],
    queryFn: async ({ signal }) => {
      // sleep
      await new Promise((resolve) => setTimeout(resolve, 500));

      if (!signal?.aborted) {
        const workflows: Workflow[] = await invoke("fetch_workflows_for_repo", {
          owner: w.owner,
          repo: w.repo,
          token: storage.getToken(),
        });

        return workflows;
      }

      return [];
    },
    enabled: w.owner !== "" && w.repo !== "",
  });

  function onSubmit(values: z.infer<typeof formSchema>) {
    props.addPr({
      status: { kind: "unknown" },
      number: values.pr,
      owner: values.owner,
      workflowId: values.workflow,
      repo: values.repo,
    });
    form.resetField("pr");
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="flex flex-col gap-8">
        <div className="flex flex-col gap-2">
          <FormField
            control={form.control}
            name="owner"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Owner</FormLabel>
                <FormControl>
                  <Input placeholder="Owner" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="repo"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Repo</FormLabel>
                <FormControl>
                  <Input placeholder="Repo" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="pr"
            render={({ field }) => (
              <FormItem>
                <FormLabel>PR #</FormLabel>
                <FormControl>
                  <Input type="number" placeholder="PR #" {...field} />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="workflow"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Workflow</FormLabel>
                <Select
                  onValueChange={field.onChange}
                  defaultValue={field.value.toString()}
                  disabled={!workflows?.length}
                >
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a workflow" />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    {(workflows || []).map((workflow) => {
                      return (
                        <SelectItem value={workflow.id.toString()}>
                          {workflow.name}
                        </SelectItem>
                      );
                    })}
                  </SelectContent>
                </Select>
                <FormMessage />
              </FormItem>
            )}
          />
        </div>
        <Button variant="secondary" type="submit">
          Submit
        </Button>
      </form>
    </Form>
  );
}
