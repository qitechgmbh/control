import { z } from "zod";
import { Icon, IconName } from "./Icon";
import React, { useEffect, useState } from "react";
import { toast } from "sonner";
import { Value } from "./Value";
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
        <Icon name={icon} size={16} />
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

function HttpErrorToast({ res }: { res: Response }) {
  const { status } = res;
  const description =
    friendlyHttpStatus[status.toString() as keyof typeof friendlyHttpStatus];

  const [body, setBody] = useState<string | null>(null);
  useEffect(() => {
    if (res.status !== 422) {
      return;
    }
    try {
      res.text().then((text) => {
        console.log(text);
        setBody(text);
      });
    } catch (e) {
      console.error(e);
    }
  }, [res]);

  return (
    <Toast title="API Fehler" icon="lu:TriangleAlert">
      <div>
        API hat Status <Value value={status} />{" "}
        {description ? `(${description}) ` : " "}
        zurückgegeben. {body}
      </div>
    </Toast>
  );
}

export function toastHttpNotOk(res: Response) {
  toast(<HttpErrorToast res={res} />);
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
