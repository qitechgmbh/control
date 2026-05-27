import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import React, { useEffect, useState } from "react";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { Alert } from "@/components/Alert";
import { Markdown } from "@/components/Markdown";
import { TouchButton } from "@/components/touch/TouchButton";
import { useUpdateStore } from "@/stores/updateStore";
import { GithubSource } from "./GithubSourceDialog";

export function ChangelogPage() {
  const navigate = useNavigate();
  const { isUpdating } = useUpdateStore();
  const search = useSearch({
    from: "/_sidebar/setup/update/changelog",
  });

  const versionType = search.branch
    ? "Branch"
    : search.tag
      ? "Tag"
      : search.commit
        ? "Commit"
        : "Unknown";

  const source: GithubSource = {
    githubRepoOwner: search.githubRepoOwner,
    githubRepoName: search.githubRepoName,
  };

  const versionName = search.branch ?? search.tag ?? search.commit;
  const ref = search.branch ?? search.tag ?? search.commit!;

  let [changelog, setChangelog] = useState<string | null | undefined>(
    undefined,
  );

  // install callback
  window.update.onFetchChangelog((result) => {
    setChangelog(result);
  });

  // Retrieve update targets
  useEffect(() => {
    window.update.fetchChangelog(source, ref);
  }, []);

  return (
    <Page>
      <SectionTitle title={`Changelog of ${versionType}`}>
        <span className="font-mono text-2xl">{versionName}</span>
      </SectionTitle>
      {changelog === undefined ? (
        <LoadingSpinner />
      ) : (
        <>
          <TouchButton
            className="w-max"
            icon="lu:Check"
            disabled={isUpdating}
            onClick={() => {
              navigate({
                to: "/_sidebar/setup/update/execute",
                search,
              });
            }}
          >
            {isUpdating ? "Update in Progress..." : "Update to This Version"}
          </TouchButton>
          {changelog === null ? (
            <Alert title="Changelog not found" variant="warning">
              The changelog file could not be found in the selected version.
            </Alert>
          ) : (
            <Markdown text={changelog} />
          )}
        </>
      )}
    </Page>
  );
}
