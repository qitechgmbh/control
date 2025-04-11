import { Icon } from "@/components/Icon";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useEffect } from "react";

// This PAT only has access to public qitechgmbh repos
export const authToken =
  "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";
const fetchOptions = {
  headers: {
    ...(authToken && { Authorization: `token ${authToken}` }),
    Accept: "application/vnd.github.v3+json",
  },
};

export function ChooseVersionPage() {
  const [commits, setCommits] = React.useState<any[]>([]);
  const [branches, setBranches] = React.useState<any[]>([]);
  const [tags, setTags] = React.useState<any[]>([]);
  useEffect(() => {
    fetch(
      `https://api.github.com/repos/qitechgmbh/control/commits`,
      fetchOptions,
    )
      .then((res) => res.json())
      .then((data) => {
        setCommits(data);
      });
  }, []);
  useEffect(() => {
    fetch(
      `https://api.github.com/repos/qitechgmbh/control/branches`,
      fetchOptions,
    )
      .then((res) => res.json())
      .then((data) => {
        setBranches(
          data
            .map((branch) => {
              // find commit for branch
              const commit = commits.find(
                (commit) => commit.sha === branch.commit.sha,
              );
              if (commit) {
                branch.date = commit.commit.author.date;
              }
              return branch;
            })
            .sort((a, b) => {
              return new Date(b.date).getTime() - new Date(a.date).getTime();
            }),
        );
      });
    fetch(`https://api.github.com/repos/qitechgmbh/control/tags`, fetchOptions)
      .then((res) => res.json())
      .then((data) => {
        setTags(
          data
            .map((tag) => {
              // find commit for tag
              const commit = commits.find(
                (commit) => commit.sha === tag.commit.sha,
              );
              if (commit) {
                tag.date = commit.commit.author.date;
              }
              return tag;
            })
            .sort((a, b) => {
              return new Date(b.date).getTime() - new Date(a.date).getTime();
            }),
        );
      });
  }, [commits]);

  // useEffect(() => {
  //   fetch(
  //     `https://api.github.com/repos/qitechgmbh/control/contents/CHANGELOG.md?ref=${ref}`,
  //     fetchOptions,
  //   )
  //     .then((res) => res.json())
  //     .then((data) => {
  //       if (data.content) {
  //         const decodedContent = atob(data.content);
  //         setReadme(decodedContent);
  //       }
  //     })
  //     .catch((err) => console.error(err));
  // }, [ref]);

  return (
    <Page>
      <SectionTitle title="Update"></SectionTitle>
      <span className="text-xl">Choose a Version</span>
      {tags.map((tag) => (
        <UpdateButton
          time={tag.date ? new Date(tag.date) : undefined}
          key={tag.name}
          title={tag.name}
          kind="tag"
          onClick={() => {
            setRev(tag.name);
          }}
        />
      ))}
      {commits.length == 0 && <>No Version</>}
      <span className="text-xl">Choose a Branch</span>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        {branches.map((branch) => (
          <UpdateButton
            time={branch.date ? new Date(branch.date) : undefined}
            key={branch.name}
            title={branch.name}
            kind="branch"
            onClick={() => {
              setRev(branch.name);
            }}
          />
        ))}
      </div>
      <span className="text-xl">Choose a Commit</span>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
        {commits.map((commit) => (
          <UpdateButton
            time={new Date(commit.commit.author.date)}
            key={commit.sha}
            title={commit.commit.message}
            kind="commit"
            onClick={() => {
              setRev(commit.sha);
            }}
          />
        ))}
      </div>
    </Page>
  );
}

type UpdateButtonProps = {
  time?: Date;
  title: string;
  kind: "tag" | "commit" | "branch";
  onClick: () => void;
};

export function UpdateButton({
  time,
  title,
  kind,
  onClick,
}: UpdateButtonProps) {
  return (
    <div
      className="flex flex-row items-center gap-2 rounded-3xl border border-gray-200 bg-white p-4 shadow"
      onClick={onClick}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <Icon
            name={
              kind === "tag"
                ? "lu:Tag"
                : kind === "branch"
                  ? "lu:GitBranch"
                  : "lu:GitCommitVertical"
            }
          />
          <span className="flex-1 truncate">{title}</span>
        </div>
        <span className="font-mono text-sm text-gray-700">
          {time ? time.toLocaleString() : "N/A"}
        </span>
      </div>
      <TouchButton className="flex-shrink-0">Select</TouchButton>
    </div>
  );
}
