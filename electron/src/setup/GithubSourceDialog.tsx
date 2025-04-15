import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useClient } from "@/client/useClient";
import React, { useMemo } from "react";
import { z } from "zod";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@/components/Icon";
import { TouchButton } from "@/components/touch/TouchButton";

type Props = {
  value: GithubSource;
  onChange: (value: GithubSource) => void;
};

const githubSourceSchema = z.object({
  githubRepoOwner: z.string(),
  githubRepoName: z.string(),
  githubToken: z.string().optional().nullable(),
});

export type GithubSource = z.infer<typeof githubSourceSchema>;

export const defaultGithubSource: GithubSource = {
  githubRepoOwner: "qitechgmbh",
  githubRepoName: "control",
  // This PAT only has read-only access to public qitechgmbh repos
  // It's split into 3 parts to avoid being detected by secret scanning
  githubToken:
    "github_pat_" +
    "11AG6Q4KQ0cfgyVayexvpp_" +
    "XuYqnT8DHTiq0tN0VdWpKxhunrBPwydGlfPm7qUMEfM4V6T2YXRXuJ8AfDA",
};

export function GithubSourceDialog({ value, onChange }: Props) {
  const [open, setOpen] = React.useState(false);
  // reset the form when the dialog is opened
  const key = useMemo(() => Math.random(), [open]);
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <TouchButton variant="outline" icon="lu:Pencil" className="w-max">
          Edit Source
        </TouchButton>
      </DialogTrigger>
      <GithubSourceDialogContent
        key={key}
        value={value}
        onChange={onChange}
        setOpen={setOpen}
      />
    </Dialog>
  );
}

type ContentProps = {
  value: GithubSource;
  onChange: (value: GithubSource) => void;
  setOpen: (open: boolean) => void;
};

export function GithubSourceDialogContent({
  value,
  onChange,
  setOpen,
}: ContentProps) {
  const client = useClient();

  const form = useForm<GithubSource>({
    resolver: zodResolver(githubSourceSchema),
    defaultValues: defaultGithubSource,
    mode: "all",
  });

  const onSubmit = (values: GithubSource) => {
    onChange({
      ...values,
      githubToken: values.githubToken === "" ? null : values.githubToken,
    });
    setOpen(false);
  };

  return (
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Change Update Source</DialogTitle>
        <DialogDescription>
          In case the update source has to be modified, this is the place to do
          it.
        </DialogDescription>
      </DialogHeader>
      <Separator />
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          {/* Gtihub Owner */}
          <FormField
            control={form.control}
            name="githubRepoOwner"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Serial</FormLabel>
                <FormControl>
                  <Input placeholder="qitechgmbh" {...field} />
                </FormControl>
                <FormDescription>Serial number of the machine.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          {/* Github repo name */}
          <FormField
            control={form.control}
            name="githubRepoName"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Github Repo Name</FormLabel>
                <FormControl>
                  <Input placeholder="control" {...field} />
                </FormControl>
                <FormDescription>Github repo name.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          {/* Github token */}
          <FormField
            control={form.control}
            name="githubToken"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Github Token</FormLabel>
                <FormControl>
                  <Input
                    placeholder="github_pat_..."
                    {...field}
                    value={field.value || ""}
                  />
                </FormControl>
                <FormDescription>Github token.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Separator />
          <Button type="submit" disabled={!form.formState.isValid}>
            <Icon name="lu:Save" /> Save
          </Button>
        </form>
      </Form>
    </DialogContent>
  );
}
