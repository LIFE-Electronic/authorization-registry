import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  useAdminPolicySet,
  useDeletePolicyFromPolicySet,
  useDeletePolicySet,
} from "../../network/policy-set";
import { PageLoadingFallback } from "../../components/page-loading-fallback";
import { CatchBoundary } from "../../components/catch-boundary";
import { Box, Typography, Stack, Button, Card } from "@mui/joy";
import { PolicyCard } from "../../components/policy-card";
import { z } from "zod";
import { ConfirmDialog } from "../../components/confirm-dialog";

function DeletePolicyModal({ deletePolicyId }: { deletePolicyId: string }) {
  const navigate = useNavigate();
  const params = Route.useParams();
  const search = Route.useSearch();

  const {
    mutateAsync: deletePolicy,
    isPending: isDeletePending,
    error: deleteError,
  } = useDeletePolicyFromPolicySet({
    policySetId: params.policySetId,
  });

  function onSubmit() {
    deletePolicy({ policyId: deletePolicyId }).then(() => {
      navigate({
        replace: true,
        to: "/policy_set/$policySetId",
        params,
        search: { ...search, delete_policy: undefined },
      });
    });
  }

  function onClose() {
    navigate({
      replace: true,
      to: "/policy_set/$policySetId",
      params,
      search: { ...search, delete_policy: undefined },
    });
  }

  return (
    <ConfirmDialog
      error={deleteError}
      isActionPending={isDeletePending}
      onSubmitText="Delete"
      onCancelText="Cancel"
      onSubmit={onSubmit}
      isOpen={Boolean(search.delete_policy)}
      onClose={onClose}
      title="Delete policy"
      description="Are you sure you want to delete this policy?"
      isDanger
    />
  );
}

function DeletePolicySetModal() {
  const navigate = useNavigate();
  const params = Route.useParams();
  const search = Route.useSearch();

  const {
    mutateAsync: deletePolicySet,
    isPending: isDeletePending,
    error: deleteError,
  } = useDeletePolicySet({
    policySetId: params.policySetId,
  });

  function onSubmit() {
    deletePolicySet().then(() => {
      navigate({
        replace: true,
        to: "/",
      });
    });
  }

  function onClose() {
    navigate({
      replace: true,
      to: "/policy_set/$policySetId",
      params,
      search: { ...search, delete_policy_set: undefined },
    });
  }

  return (
    <ConfirmDialog
      error={deleteError}
      isActionPending={isDeletePending}
      onSubmitText="Delete"
      onCancelText="Cancel"
      onSubmit={onSubmit}
      isOpen={Boolean(search.delete_policy_set)}
      onClose={onClose}
      title="Delete policy"
      description="Are you sure you want to delete this policy?"
      isDanger
    />
  );
}

const searchSchema = z.object({
  add_policy: z.boolean().optional(),
  delete_policy: z.string().optional(),
  delete_policy_set: z.boolean().optional(),
});

export const Route = createFileRoute("/__auth/policy_set/$policySetId/")({
  component: Component,
  errorComponent: CatchBoundary,
  validateSearch: searchSchema,
});

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();
  const search = Route.useSearch();
  const { policySetId } = Route.useParams();

  const { data: policySet, isLoading } = useAdminPolicySet({
    policySetId,
  });

  const deletePolicyId = search.delete_policy;

  return (
    <PageLoadingFallback isLoading={isLoading}>
      {policySet && (
        <>
          {deletePolicyId !== undefined && (
            <DeletePolicyModal deletePolicyId={deletePolicyId} />
          )}

          <DeletePolicySetModal />

          <Stack spacing={3}>
            <Card>
              <Stack direction="row" spacing={2}>
                <Box>
                  <Typography level="title-md">Access subject</Typography>
                  <Typography>{policySet.access_subject}</Typography>
                </Box>
                <Box>
                  <Typography level="title-md">Policy issuer</Typography>
                  <Typography>{policySet.policy_issuer}</Typography>
                </Box>
              </Stack>
            </Card>

            <Box>
              <Card>
                <Typography level="title-md">Policies</Typography>
                <Stack spacing={1} direction="row" flexWrap="wrap" useFlexGap>
                  {policySet.policies.map((p) => (
                    <PolicyCard
                      policy={p}
                      key={p.id}
                      actions={
                        <Stack spacing={1} direction="row">
                          <Button
                            onClick={() =>
                              navigate({
                                to: "/policy_set/$policySetId",
                                params,
                                search: { ...search, delete_policy: p.id },
                              })
                            }
                            color="danger"
                            variant="outlined"
                          >
                            Delete
                          </Button>
                          <Button variant="outlined">Edit</Button>
                        </Stack>
                      }
                    />
                  ))}
                </Stack>
              </Card>

              <Stack padding={2} direction="row" spacing={1}>
                <Button
                  size="lg"
                  onClick={() =>
                    navigate({
                      to: "/policy_set/$policySetId/add_policy/step1",
                      params: { policySetId },
                    })
                  }
                >
                  Add policy
                </Button>
                <Button
                  size="lg"
                  color="danger"
                  onClick={() =>
                    navigate({
                      to: "/policy_set/$policySetId",
                      params,
                      search: { ...search, delete_policy_set: true },
                    })
                  }
                >
                  Delete policy set
                </Button>
              </Stack>
            </Box>
          </Stack>
        </>
      )}
    </PageLoadingFallback>
  );
}
