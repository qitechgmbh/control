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
import { Collapsible, CollapsibleTrigger } from "@radix-ui/react-collapsible";
import { CollapsibleContent } from "@/components/ui/collapsible";

export function ChooseVersionPage() {
  const navigate = useNavigate();

  // load environment info
  const [environmentInfo, setEnvironmentInfo] = useState<
    EnvironmentInfo | undefined
  >(undefined);
  useEffectAsync(async () => {
    const _environmentInfo = await window.environment.getInfo();
    setEnvironmentInfo(_environmentInfo);
  }, []);

  const currentTimestamp = environmentInfo?.qitechOsGitTimestamp
    ? new Date(environmentInfo.qitechOsGitTimestamp).getTime()
    : 0;

  const [masterCommits, setMasterCommits] = useState<any[] | undefined>(
    undefined,
  );
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

  // Fetch master commits
  useEffect(() => {
    setMasterCommits(undefined);
    fetch(githubApiUrl + `/commits`, fetchOptions)
      .then(async (res) => {
        const json = await res.json();
        setMasterCommits(json);
      })
      .catch(() => {
        setMasterCommits([]);
      });
  }, [githubSource]);

  // Fetch branches with their commit data
  useEffect(() => {
    setBranches(undefined);
    fetch(githubApiUrl + `/branches`, fetchOptions)
      .then(async (res) => {
        const json = await res.json();

        // Fetch commit data for each branch
        const branchesWithCommitData = await Promise.all(
          json.map(async (branch) => {
            try {
              // Fetch the commit data for this branch
              const commitRes = await fetch(
                `${githubApiUrl}/commits/${branch.commit.sha}`,
                fetchOptions,
              );
              const commitData = await commitRes.json();

              // Add the date to the branch object
              if (commitData && commitData.commit) {
                branch.date = commitData.commit.author.date;
              }
            } catch (error) {
              console.error(
                `Error fetching commit for branch ${branch.name}:`,
                error,
              );
            }
            return branch;
          }),
        );

        // Sort branches by date
        setBranches(
          branchesWithCommitData.sort((a, b) => {
            return (
              new Date(b.date || 0).getTime() - new Date(a.date || 0).getTime()
            );
          }),
        );
      })
      .catch((error) => {
        console.error("Error fetching branches:", error);
        setBranches([]);
      });
  }, [githubSource]);

  // Fetch tags with their commit data
  useEffect(() => {
    setTags(undefined);
    fetch(githubApiUrl + `/tags`, fetchOptions)
      .then(async (res) => {
        const json = await res.json();

        // Fetch commit data for each tag
        const tagsWithCommitData = await Promise.all(
          json.map(async (tag) => {
            try {
              // Fetch the commit data for this tag
              const commitRes = await fetch(
                `${githubApiUrl}/commits/${tag.commit.sha}`,
                fetchOptions,
              );
              const commitData = await commitRes.json();

              // Add the date to the tag object
              if (commitData && commitData.commit) {
                tag.date = commitData.commit.author.date;
              }
            } catch (error) {
              console.error(
                `Error fetching commit for tag ${tag.name}:`,
                error,
              );
            }
            return tag;
          }),
        );

        // Sort tags by date
        setTags(
          tagsWithCommitData.sort((a, b) => {
            return (
              new Date(b.date || 0).getTime() - new Date(a.date || 0).getTime()
            );
          }),
        );
      })
      .catch((error) => {
        console.error("Error fetching tags:", error);
        setTags([]);
      });
  }, [githubSource]);

  const isOlderThanCurrent = (date?: string | Date) => {
    if (!date || !currentTimestamp) return false;
    const timestamp = new Date(date).getTime();
    return timestamp <= currentTimestamp;
  };

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
              isOlder={isOlderThanCurrent(tag.date)}
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

      <Collapsible>
        <CollapsibleTrigger>
          <div className="flex flex-row items-center gap-2">
            <span className="text-xl">Choose a Branch</span>
            <Icon name="lu:ChevronsUpDown" />
          </div>
        </CollapsibleTrigger>
        <CollapsibleContent className="pt-6">
          {branches !== undefined && branches.length > 0 ? (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {branches.map((branch) => (
                <UpdateButton
                  time={branch.date ? new Date(branch.date) : undefined}
                  key={branch.name}
                  title={branch.name}
                  kind="branch"
                  isOlder={isOlderThanCurrent(branch.date)}
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
        </CollapsibleContent>
      </Collapsible>

      <Collapsible>
        <CollapsibleTrigger>
          <div className="flex flex-row items-center gap-2">
            <span className="text-xl">Choose a Master Commit</span>
            <Icon name="lu:ChevronsUpDown" />
          </div>
        </CollapsibleTrigger>
        <CollapsibleContent className="pt-6">
          {masterCommits !== undefined && masterCommits.length > 0 ? (
            <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
              {masterCommits.map((commit) => (
                <UpdateButton
                  time={new Date(commit.commit.author.date)}
                  key={commit.sha}
                  title={commit.commit.message}
                  kind="commit"
                  isOlder={isOlderThanCurrent(commit.commit.author.date)}
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
          {masterCommits === undefined && <LoadingSpinner />}
          {masterCommits?.length == 0 && <>No Master Commits</>}
        </CollapsibleContent>
      </Collapsible>
    </Page>
  );
}

type UpdateButtonProps = {
  time?: Date;
  title: string;
  kind: "tag" | "commit" | "branch";
  isOlder?: boolean;
  onClick: () => void;
};

export function UpdateButton({
  time,
  title,
  kind,
  onClick,
  isOlder = false,
}: UpdateButtonProps) {
  return (
    <div
      className={`flex flex-row items-center gap-2 rounded-3xl border border-gray-200 ${
        isOlder ? "bg-gray-100" : "bg-white"
      } p-4 shadow`}
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
            className={isOlder ? "text-gray-400" : ""}
          />
          <span className={`flex-1 truncate ${isOlder ? "text-gray-400" : ""}`}>
            {title}
            {isOlder && " (older)"}
          </span>
        </div>
        <span
          className={`font-mono text-sm ${isOlder ? "text-gray-400" : "text-gray-700"}`}
        >
          {time ? time.toLocaleString() : "N/A"}
        </span>
      </div>
      <TouchButton
        className="flex-shrink-0"
        variant={isOlder ? "outline" : "default"}
        onClick={onClick}
      >
        Select
      </TouchButton>
    </div>
  );
}

export function CurrentVersionCard() {
  const navigate = useNavigate();

  const [environmentInfo, setEnvironmentInfo] = useState<
    EnvironmentInfo | undefined
  >(undefined);
  useEffectAsync(async () => {
    const _environmentInfo = await window.environment.getInfo();
    setEnvironmentInfo(_environmentInfo);
  }, []);

  const githubRegex =
    /https:\/\/(?<token>[^@.]+)@?github\.com\/(?<username>[^/^.]+)\/(?<repository>[^/^.]+)(?:.+)/;
  const match = environmentInfo?.qitechOsGitUrl?.match(githubRegex);
  const githubRepoOwner = match?.groups?.username;
  const githubRepoName = match?.groups?.repository;

  const urlWithCensoredToken = environmentInfo?.qitechOsGitUrl?.replace(
    githubRegex,
    `https://github.com/${githubRepoOwner}/${githubRepoName}`,
  );

  if (!environmentInfo) {
    return <LoadingSpinner />;
  }

  return (
    <div className="flex w-max items-center gap-4 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="flex min-w-0 flex-1 flex-col">
        <div className="flex items-center gap-2">
          <Icon name="lu:Tag" />
          <span className="flex-1 truncate">
            {environmentInfo?.qitechOsGitAbbreviation}
          </span>
        </div>
        <span className="font-mono text-sm text-gray-700">
          {environmentInfo?.qitechOsGitTimestamp
            ? new Date(environmentInfo?.qitechOsGitTimestamp).toLocaleString()
            : "N/A"}
        </span>
        <span className="font-mono text-sm text-gray-700">
          {environmentInfo?.qitechOsGitCommit ?? "N/A"}
        </span>
        <span className="font-mono text-sm text-gray-700">
          {urlWithCensoredToken ?? "N/A"}
        </span>
      </div>
      <TouchButton
        className="flex-shrink-0"
        onClick={() => {
          if (!githubRepoOwner || !githubRepoName) {
            console.error(
              "GitHub repo owner or name not found in environment info.",
            );
            return;
          }
          navigate({
            to: "/_sidebar/setup/update/changelog",
            search: {
              commit: environmentInfo?.qitechOsGitCommit,
              tag: undefined,
              branch: undefined,
              githubRepoOwner: githubRepoOwner,
              githubRepoName: githubRepoName,
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
