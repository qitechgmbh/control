import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
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
import { Badge } from "@/components/ui/badge";

type Props = {
  value: GithubSource;
  onChange: (value: GithubSource) => void;
};

export const githubSourceSchema = z.object({
  githubRepoOwner: z.string().catch("qitechgmbh"),
  githubRepoName: z.string().catch("control"),
});

export type GithubSource = z.infer<typeof githubSourceSchema>;

export const defaultGithubSource: GithubSource = {
  githubRepoOwner: "qitechgmbh",
  githubRepoName: "control",
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
  const form = useForm<GithubSource>({
    resolver: zodResolver(githubSourceSchema),
    defaultValues: value,
    mode: "all",
  });

  const onSubmit = (values: GithubSource) => {
    onChange({
      ...values,
    });
    setOpen(false);
  };

  // Token state
  const [tokenSet, setTokenSet] = React.useState(false);
  const [tokenInput, setTokenInput] = React.useState("");
  const [tokenLoading, setTokenLoading] = React.useState(false);
  const [tokenError, setTokenError] = React.useState<string | null>(null);

  React.useEffect(() => {
    window.update.hasToken().then(setTokenSet);
  }, []);

  const handleSaveToken = async () => {
    setTokenLoading(true);
    setTokenError(null);
    const result = await window.update.saveToken(tokenInput.trim());
    setTokenLoading(false);
    if (result.success) {
      setTokenSet(tokenInput.trim().length > 0);
      setTokenInput("");
    } else {
      setTokenError(result.error ?? "Failed to save token");
    }
  };

  const handleLoadFromUsb = async () => {
    setTokenLoading(true);
    setTokenError(null);
    const result = await window.update.loadTokenFromFile();
    setTokenLoading(false);
    if (result.success) {
      setTokenSet(true);
    } else if (result.error !== "Cancelled") {
      setTokenError(result.error ?? "Failed to load token from file");
    }
  };

  const handleClearToken = async () => {
    await window.update.clearToken();
    setTokenSet(false);
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
          {/* GitHub Owner */}
          <FormField
            control={form.control}
            name="githubRepoOwner"
            render={({ field }) => (
              <FormItem>
                <FormLabel>GitHub Repository Owner</FormLabel>
                <FormControl>
                  <Input placeholder="qitechgmbh" {...field} />
                </FormControl>
                <FormDescription>
                  Name of the GitHub repository owner.
                </FormDescription>
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
          <Separator />
          <Button type="submit" disabled={!form.formState.isValid}>
            <Icon name="lu:Save" /> Save
          </Button>
        </form>
      </Form>

      {/* ── GitHub Access Token ──────────────────────────────────────────── */}
      <Separator />
      <div className="space-y-3">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">GitHub Access Token</span>
          {tokenSet ? (
            <Badge variant="default" className="text-xs">
              Token set
            </Badge>
          ) : (
            <Badge variant="secondary" className="text-xs">
              No token
            </Badge>
          )}
        </div>
        <p className="text-muted-foreground text-xs">
          Required only for private repositories. The token is encrypted and
          stored on disk.
        </p>

        {tokenSet ? (
          <Button
            type="button"
            variant="destructive"
            size="sm"
            onClick={handleClearToken}
            disabled={tokenLoading}
          >
            <Icon name="lu:Trash2" /> Remove token
          </Button>
        ) : (
          <div className="space-y-2">
            {/* Manual entry */}
            <div className="flex gap-2">
              <Input
                type="password"
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
                value={tokenInput}
                onChange={(e) => setTokenInput(e.target.value)}
                className="font-mono text-xs"
              />
              <Button
                type="button"
                size="sm"
                onClick={handleSaveToken}
                disabled={tokenLoading || tokenInput.trim().length === 0}
              >
                <Icon name="lu:Save" /> Save
              </Button>
            </div>
            {/* Load from USB */}
            <TouchButton
              type="button"
              variant="outline"
              icon="lu:Usb"
              className="w-max"
              onClick={handleLoadFromUsb}
              disabled={tokenLoading}
            >
              Load from USB
            </TouchButton>
          </div>
        )}

        {tokenError && <p className="text-destructive text-xs">{tokenError}</p>}
      </div>
    </DialogContent>
  );
}
