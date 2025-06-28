import { Page } from "@/components/Page";
import { SectionTitle } from "@/components/SectionTitle";
import React, { useEffect } from "react";
import { useNavigate, useSearch } from "@tanstack/react-router";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { Alert } from "@/components/Alert";
import { Markdown } from "@/components/Markdown";
import { TouchButton } from "@/components/touch/TouchButton";
import { useUpdateStore } from "@/stores/updateStore";

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

  const versionName = search.branch ?? search.tag ?? search.commit;

  const githubApiUrl = `https://api.github.com/repos/${search.githubRepoOwner}/${search.githubRepoName}`;

  const fetchOptions = {
    headers: {
      ...(search.githubToken && {
        Authorization: `token ${search.githubToken}`,
      }),
      Accept: "application/vnd.github.v3+json",
    },
  };

  const [changelog, setReadme] = React.useState<string | undefined | null>(
    undefined,
  );

  useEffect(() => {
    const params = new URLSearchParams();
    if (search.branch) {
      params.append("ref", search.branch);
    } else if (search.tag) {
      params.append("ref", search.tag);
    } else if (search.commit) {
      params.append("ref", search.commit);
    }
    fetch(`${githubApiUrl}/contents/CHANGELOG.md?${params}`, fetchOptions)
      .then((res) => res.json())
      .then((data) => {
        if (data.content) {
          const decodedContent = atob(data.content);
          setReadme(decodedContent);
        }
      })
      .catch((err) => console.error(err));
  }, [search]);

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
