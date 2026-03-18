import { z } from "zod";
import { Icon, IconName } from "./Icon";
import React from "react";
import { toast } from "sonner";
import { fromError } from "zod-validation-error";

type Props = {
  title: string;
};

type ZodErrorToastProps = {
  error: z.ZodError;
} & Props;

type ToastContentProps = {
  title: string;
  icon?: IconName;
  children: React.ReactNode;
};

export function Toast({ title, children, icon }: ToastContentProps) {
  return (
    <div className="flex flex-col gap-2 p-2">
      <div className="flex flex-row gap-2">
        {icon && <Icon name={icon} />}
        <div className="font-bold">{title}</div>
      </div>
      {children}
    </div>
  );
}

export function ZodErrorToast({ error, title }: ZodErrorToastProps) {
  const readable = fromError(error, {
    prefix: "",
    prefixSeparator: "",
    maxIssuesInMessage: 1,
    includePath: true,
  });
  return (
    <Toast title={title} icon="lu:TriangleAlert">
      {readable.message}
    </Toast>
  );
}

export function toastZodError(error: z.ZodError, title: string) {
  toast(<ZodErrorToast error={error} title={title} />);
}

function HttpErrorToast({
  status,
  error,
}: {
  status: number;
  error: string | undefined;
}) {
  const description =
    friendlyHttpStatus[status.toString() as keyof typeof friendlyHttpStatus];

  return (
    <Toast
      title={`API Fehler ${status} ${description ? `(${description}) ` : ""}`}
      icon="lu:TriangleAlert"
    >
      <div className="text-zinc-500">{error}</div>
    </Toast>
  );
}

export function toastHttpNotOk(status: number, error: string | undefined) {
  toast(<HttpErrorToast status={status} error={error} />);
}

export function ErrorToast({ title, error }: Props & { error: string }) {
  return (
    <Toast title={title} icon="lu:TriangleAlert">
      <div className="text-zinc-500">{error}</div>
    </Toast>
  );
}

export function toastError(title: string, error: string) {
  toast(<ErrorToast title={title} error={error} />);
}

const friendlyHttpStatus = {
  "200": "OK",
  "201": "Created",
  "202": "Accepted",
  "203": "Non-Authoritative Information",
  "204": "No Content",
  "205": "Reset Content",
  "206": "Partial Content",
  "300": "Multiple Choices",
  "301": "Moved Permanently",
  "302": "Found",
  "303": "See Other",
  "304": "Not Modified",
  "305": "Use Proxy",
  "306": "Unused",
  "307": "Temporary Redirect",
  "400": "Bad Request",
  "401": "Unauthorized",
  "402": "Payment Required",
  "403": "Forbidden",
  "404": "Not Found",
  "405": "Method Not Allowed",
  "406": "Not Acceptable",
  "407": "Proxy Authentication Required",
  "408": "Request Timeout",
  "409": "Conflict",
  "410": "Gone",
  "411": "Length Required",
  "412": "Precondition Required",
  "413": "Request Entry Too Large",
  "414": "Request-URI Too Long",
  "415": "Unsupported Media Type",
  "416": "Requested Range Not Satisfiable",
  "417": "Expectation Failed",
  "418": "I'm a teapot",
  "429": "Too Many Requests",
  "500": "Internal Server Error",
  "501": "Not Implemented",
  "502": "Bad Gateway",
  "503": "Service Unavailable",
  "504": "Gateway Timeout",
  "505": "HTTP Version Not Supported",
};
