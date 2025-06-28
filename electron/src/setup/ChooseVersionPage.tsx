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
import {
  listNixOSGenerations,
  setNixOSGeneration,
  deleteNixOSGeneration,
  isNixOSAvailable,
} from "@/helpers/nixos_helpers";
import { useUpdateStore } from "@/stores/updateStore";

export function ChooseVersionPage() {
  const navigate = useNavigate();
  const { isUpdating, currentUpdateInfo } = useUpdateStore();

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

  // NixOS generations state
  const [nixosGenerations, setNixosGenerations] = useState<
    NixOSGeneration[] | undefined
  >(undefined);
  const [nixosError, setNixosError] = useState<string | undefined>(undefined);
  const [generationActionLoading, setGenerationActionLoading] = useState<
    string | null
  >(null);

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

  // Fetch NixOS generations
  useEffectAsync(async () => {
    if (!isNixOSAvailable()) {
      setNixosGenerations([]);
      return;
    }

    try {
      const result = await listNixOSGenerations();
      if (result.success) {
        setNixosGenerations(result.generations);
        setNixosError(undefined);
      } else {
        setNixosError(result.error || "Failed to fetch NixOS generations");
        setNixosGenerations([]);
      }
    } catch (error) {
      setNixosError(error instanceof Error ? error.message : String(error));
      setNixosGenerations([]);
    }
  }, []);

  const isOlderThanCurrent = (date?: string | Date) => {
    if (!date || !currentTimestamp) return false;
    const timestamp = new Date(date).getTime();
    return timestamp <= currentTimestamp;
  };

  // NixOS generation handlers
  const handleSetGeneration = async (generationId: string) => {
    setGenerationActionLoading(generationId);
    try {
      const result = await setNixOSGeneration(generationId);
      if (result.success) {
        // Refresh the generations list to show updated current generation
        const updatedResult = await listNixOSGenerations();
        if (updatedResult.success) {
          setNixosGenerations(updatedResult.generations);
        }
        console.log(`Successfully set generation ${generationId}`);
      } else {
        console.error(`Failed to set generation: ${result.error}`);
        setNixosError(result.error || "Failed to set generation");
      }
    } catch (error) {
      console.error("Error setting generation:", error);
      setNixosError(error instanceof Error ? error.message : String(error));
    } finally {
      setGenerationActionLoading(null);
    }
  };

  const handleDeleteGeneration = async (generationId: string) => {
    if (
      !confirm(
        `Are you sure you want to delete generation ${generationId}? This will also update the bootloader menu. This action cannot be undone.`,
      )
    ) {
      return;
    }

    setGenerationActionLoading(generationId);
    try {
      const result = await deleteNixOSGeneration(generationId);
      if (result.success) {
        // Refresh the generations list
        const updatedResult = await listNixOSGenerations();
        if (updatedResult.success) {
          setNixosGenerations(updatedResult.generations);
        }
        console.log(
          `Successfully deleted generation ${generationId} and updated bootloader`,
        );
      } else {
        console.error(`Failed to delete generation: ${result.error}`);
        setNixosError(result.error || "Failed to delete generation");
      }
    } catch (error) {
      console.error("Error deleting generation:", error);
      setNixosError(error instanceof Error ? error.message : String(error));
    } finally {
      setGenerationActionLoading(null);
    }
  };

  return (
    <Page>
      <SectionTitle title="Current Version"></SectionTitle>
      <CurrentVersionCard />

      {/* Current Update Status */}
      {isUpdating && currentUpdateInfo && (
        <>
          <SectionTitle title="Update in Progress"></SectionTitle>
          <div className="mb-4 rounded-lg border border-blue-200 bg-blue-50 p-4">
            <div className="mb-3 flex items-center gap-2">
              <LoadingSpinner />
              <span className="font-semibold text-blue-800">
                Updating System...
              </span>
            </div>
            <div className="space-y-1 text-sm text-blue-700">
              <div>
                <span className="font-medium">Repository:</span>{" "}
                <span className="font-mono">
                  {currentUpdateInfo.githubRepoOwner}/
                  {currentUpdateInfo.githubRepoName}
                </span>
              </div>
              {currentUpdateInfo.tag && (
                <div>
                  <span className="font-medium">Tag:</span>{" "}
                  <span className="font-mono">{currentUpdateInfo.tag}</span>
                </div>
              )}
              {currentUpdateInfo.branch && (
                <div>
                  <span className="font-medium">Branch:</span>{" "}
                  <span className="font-mono">{currentUpdateInfo.branch}</span>
                </div>
              )}
              {currentUpdateInfo.commit && (
                <div>
                  <span className="font-medium">Commit:</span>{" "}
                  <span className="font-mono">
                    {currentUpdateInfo.commit.substring(0, 8)}
                  </span>
                </div>
              )}
            </div>
            <TouchButton
              className="mt-3 w-max"
              onClick={() => {
                navigate({
                  to: "/_sidebar/setup/update/execute",
                  search: {
                    githubRepoOwner: currentUpdateInfo.githubRepoOwner,
                    githubRepoName: currentUpdateInfo.githubRepoName,
                    githubToken: currentUpdateInfo.githubToken,
                    tag: currentUpdateInfo.tag,
                    branch: currentUpdateInfo.branch,
                    commit: currentUpdateInfo.commit,
                  },
                });
              }}
            >
              View Update Progress
            </TouchButton>
          </div>
        </>
      )}

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

      <SectionTitle title="Installed Versions"></SectionTitle>

      {nixosError && (
        <span className="w-max">
          <Alert title="Error" variant="error">
            {nixosError}
          </Alert>
        </span>
      )}

      {nixosGenerations !== undefined && nixosGenerations.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {nixosGenerations.map((generation) => (
            <GenerationButton
              key={generation.id}
              generation={generation}
              isLoading={generationActionLoading === generation.id}
              onSet={() => handleSetGeneration(generation.id)}
              onDelete={() => handleDeleteGeneration(generation.id)}
            />
          ))}
        </div>
      ) : null}
      {nixosGenerations === undefined && isNixOSAvailable() && (
        <LoadingSpinner />
      )}
      {nixosGenerations?.length === 0 && <>No NixOS generations found</>}
      {!isNixOSAvailable() && (
        <span className="w-max">
          <Alert title="NixOS Not Available" variant="warning">
            NixOS generation management is not available on this system.
          </Alert>
        </span>
      )}
    </Page>
  );
}

type GenerationButtonProps = {
  generation: NixOSGeneration;
  isLoading: boolean;
  onSet: () => void;
  onDelete: () => void;
};

function GenerationButton({
  generation,
  isLoading,
  onSet,
  onDelete,
}: GenerationButtonProps) {
  return (
    <div className="flex flex-row items-center gap-2 rounded-3xl border border-gray-200 bg-white p-4 shadow">
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <Icon name="lu:History" />
          <span className="flex-1 truncate">
            Update {generation.id}
            {generation.current && " (current)"}
          </span>
        </div>
        <span className="block truncate font-mono text-sm text-gray-700">
          {generation.name}
        </span>
        <span className="block font-mono text-sm text-gray-700">
          {generation.date ? new Date(generation.date).toLocaleString() : "N/A"}
        </span>
        {generation.kernelVersion && (
          <span className="font-mono text-sm text-gray-600">
            Kernel: {generation.kernelVersion}
          </span>
        )}
        {generation.description && (
          <span className="text-sm text-gray-600">
            {generation.description}
          </span>
        )}
      </div>
      <div className="flex gap-2">
        <TouchButton
          className="flex-shrink-0"
          variant="outline"
          onClick={onSet}
          disabled={isLoading || generation.current}
        >
          {isLoading ? <LoadingSpinner /> : "Select"}
        </TouchButton>
        <TouchButton
          className="flex-shrink-0"
          variant="destructive"
          onClick={onDelete}
          disabled={isLoading || generation.current}
        >
          {isLoading ? <LoadingSpinner /> : "Delete"}
        </TouchButton>
      </div>
    </div>
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
