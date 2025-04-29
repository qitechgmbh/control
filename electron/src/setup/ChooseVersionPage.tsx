import { Icon } from "@/components/Icon";
import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import { TouchButton } from "@/components/touch/TouchButton";
import React, { useEffect, useState } from "react";
import {
  defaultGithubSource,
  GithubSource,
  GithubSourceDialog,
} from "./GithubSourceDialog";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { Alert } from "@/components/Alert";
import { useNavigate } from "@tanstack/react-router";
import { useEffectAsync } from "@/lib/useEffectAsync";

export function ChooseVersionPage() {
  const navigate = useNavigate();

  // load environment info
  const [environmentInfo, setEnvironmentInfo] = useState<
    EnvironmentInfo | undefined
  >();
  useEffectAsync(async () => {
    const _environmentInfo = await window.environment.getInfo();
    setEnvironmentInfo(_environmentInfo);
  }, []);

  const [commits, setCommits] = useState<any[] | undefined>(undefined);
  const [branches, setBranches] = useState<any[] | undefined>(undefined);
  const [tags, setTags] = useState<any[] | undefined>(undefined);

  const [githubSource, setGithubSource] =
    useState<GithubSource>(defaultGithubSource);

  const githubApiUrl = `https://api.github.com/repos/${githubSource.githubRepoOwner}/${githubSource.githubRepoName}`;

  const fetchOptions = {
    headers: {
      ...(githubSource.githubToken && {
        Authorization: `token ${githubSource.githubToken}`,
      }),
      Accept: "application/vnd.github.v3+json",
    },
  };

  useEffect(() => {
    setCommits(undefined);
    fetch(githubApiUrl + `/commits`, fetchOptions)
      .then(async (res) => {
        const json = await res.json();
        setCommits(json);
      })
      .catch(() => {
        setCommits([]);
      });
  }, [githubSource]);

  useEffect(() => {
    setBranches(undefined);
    fetch(githubApiUrl + `/branches`, fetchOptions)
      .then(async (res) => {
        const json = await res.json();
        setBranches(
          json
            .map((branch) => {
              // find commit for branch
              const commit = commits?.find(
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
      })
      .catch(() => {
        setBranches([]);
      });
    setTags(undefined);
    fetch(githubApiUrl + `/tags`, fetchOptions)
      .then(async (res) => {
        const json = await res.json();
        setTags(
          json
            .map((tag) => {
              // find commit for tag
              const commit = commits?.find(
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
      })
      .catch(() => {
        setTags([]);
      });
  }, [commits, githubSource]);

  return (
    <Page>
      <SectionTitle title="Current Version"></SectionTitle>
      <CurrentVersionCard />
      <SectionTitle title="Update"></SectionTitle>
      <div className="flex flex-row items-center gap-4">
        <div className="flex flex-col">
          Update source:
          <a className="font-mono text-blue-500">
            {`https://github.com/${githubSource.githubRepoOwner}/${
              githubSource.githubRepoName
            }`}
          </a>
        </div>
        <GithubSourceDialog value={githubSource} onChange={setGithubSource} />
      </div>
      <span className="w-max">
        <Alert title="Internet Access Needed" variant="info">
          You must connect to the internet to fetch the latest versions and
          update the system.
        </Alert>
      </span>
      <span className="text-xl">Choose a Version</span>
      {tags !== undefined && tags.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {tags.map((tag) => (
            <UpdateButton
              time={tag.date ? new Date(tag.date) : undefined}
              key={tag.name}
              title={tag.name}
              kind="tag"
              onClick={() => {
                navigate({
                  to: "/_sidebar/setup/update/changelog",
                  search: {
                    tag: tag.name,
                    ...githubSource,
                  },
                });
              }}
            />
          ))}
        </div>
      ) : null}
      {tags === undefined && <LoadingSpinner />}
      {tags?.length == 0 && <>No Versions</>}

      <span className="text-xl">Choose a Branch</span>
      {branches !== undefined && branches.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {branches.map((branch) => (
            <UpdateButton
              time={branch.date ? new Date(branch.date) : undefined}
              key={branch.name}
              title={branch.name}
              kind="branch"
              onClick={() => {
                navigate({
                  to: "/_sidebar/setup/update/changelog",
                  search: {
                    branch: branch.name,
                    ...githubSource,
                  },
                });
              }}
            />
          ))}
        </div>
      ) : null}
      {branches === undefined && <LoadingSpinner />}
      {branches?.length == 0 && <>No Branches</>}

      <span className="text-xl">Choose a Commit</span>
      {commits !== undefined && commits.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {commits.map((commit) => (
            <UpdateButton
              time={new Date(commit.commit.author.date)}
              key={commit.sha}
              title={commit.commit.message}
              kind="commit"
              onClick={() => {
                navigate({
                  to: "/_sidebar/setup/update/changelog",
                  search: {
                    commit: commit.sha,
                    ...githubSource,
                  },
                });
              }}
            />
          ))}
        </div>
      ) : null}
      {commits === undefined && <LoadingSpinner />}
      {commits?.length == 0 && <>No Commits</>}
    </Page>
  );
}

type UpdateButtonProps = {
  time?: Date;
  title: string;
  kind: "tag" | "commit" | "branch";
  isTooOld?: boolean;
  onClick: () => void;
};

export function UpdateButton({
  time,
  title,
  kind,
  onClick,
  isTooOld = false,
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
      <TouchButton className="flex-shrink-0" disabled={isTooOld}>
        Select
      </TouchButton>
    </div>
  );
}

export function CurrentVersionCard() {
  const navigate = useNavigate();

  const [environmentInfo, setEnvironmentInfo] = useState<
    EnvironmentInfo | undefined
  >({
    qitechOs: false,
    qitechOsGitTimestamp: new Date(),
    qitechOsGitCommit: "alksndlasdnoqweidn",
    qitechOsGitAbbrevation: "fix/45",
  });

  const githubRegex = /https:\/\/.+github\.com\/([^\/^\.]+)\/([^\/^\.]+)(?:.+)/;
  const match = githubRegex.exec(environmentInfo?.qitechOsGitUrl ?? "");

  useEffectAsync(async () => {
    const _environmentInfo = await window.environment.getInfo();
    setEnvironmentInfo(_environmentInfo);
  }, []);

  if (!environmentInfo) {
    return <LoadingSpinner />;
  }

  return (
    <div className="flex w-max items-center gap-2 gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex items-center gap-2">
          <Icon name="lu:Tag" />
          <span className="flex-1 truncate">
            {environmentInfo?.qitechOsGitAbbrevation}
          </span>
        </div>
        <span className="font-mono text-sm text-gray-700">
          {environmentInfo?.qitechOsGitTimestamp
            ? environmentInfo?.qitechOsGitTimestamp.toLocaleString()
            : "N/A"}
        </span>
        <span className="font-mono text-sm text-gray-700">
          {environmentInfo?.qitechOsGitCommit ?? "N/A"}
        </span>
        <span className="font-mono text-sm text-gray-700">
          {environmentInfo?.qitechOsGitUrl ?? "N/A"}
        </span>
      </div>
      <TouchButton
        className="flex-shrink-0"
        onClick={() => {
          navigate({
            to: "/_sidebar/setup/update/changelog",
            search: {
              commit: environmentInfo?.qitechOsGitCommit,
              tag: undefined,
              branch: undefined,
              githubRepoOwner: match?.[1]!,
              githubRepoName: match?.[2]!,
              githubToken: undefined,
            },
          });
        }}
      >
        Changelog
      </TouchButton>
    </div>
  );
}
