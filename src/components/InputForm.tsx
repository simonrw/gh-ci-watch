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

const formSchema = z.object({
  owner: z.string().min(1).max(50, {
    message: "Owner must be less than 50 characters",
  }),
  repo: z.string().min(1).max(50, {
    message: "Repo must be less than 50 characters",
  }),
  pr: z.coerce.number().min(0),
});

type InputFormProps = {
  addPr: (pr: Pr) => void;
};

export function InputForm(props: InputFormProps) {
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      owner: "",
      repo: "",
      pr: 0,
    },
  });

  function onSubmit(values: z.infer<typeof formSchema>) {
    props.addPr({
      status: "Unknown",
      number: values.pr,
      owner: values.owner,
      repo: values.repo,
    });
    form.resetField("pr");
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
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
        <Button variant="secondary" type="submit">
          Submit
        </Button>
      </form>
    </Form>
  );
}
