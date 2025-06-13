import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  useAdminPolicySet,
  useDeleteAdminPolicyFromPolicySet,
  useDeleteAdminPolicySet,
} from "@/network/policy-set";
import { CatchBoundary } from "@/components/catch-boundary";
import { PolicySetDetail } from "@/components/policy-set-detail-page";

export const Route = createFileRoute("/__auth/admin/policy_set/$policySetId/")({
  component: Component,
  errorComponent: CatchBoundary,
});

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();
  const { data: policySet, isLoading } = useAdminPolicySet({
    policySetId: params.policySetId,
  });
  const {
    mutateAsync: deletePolicySet,
    isPending: isDeletePending,
    error: deleteError,
  } = useDeleteAdminPolicySet({
    policySetId: params.policySetId,
  });

  function onDeletePolicySet() {
    deletePolicySet().then(() => {
      navigate({
        replace: true,
        to: "/admin",
      });
    });
  }

  const {
    mutateAsync: deletePolicy,
    isPending: isDeletePolicyPending,
    error: deletePolicyError,
  } = useDeleteAdminPolicyFromPolicySet({
    policySetId: params.policySetId,
  });

  function onDeletePolicy(policyId: string) {
    return deletePolicy({ policyId });
  }

  return (
    <PolicySetDetail
      deletePolicyPending={isDeletePolicyPending}
      deletePolicyError={deletePolicyError}
      onDeletePolicy={onDeletePolicy}
      deletePolicySetPending={isDeletePending}
      deletePolicySetError={deleteError}
      onDeletePolicySet={onDeletePolicySet}
      policySet={policySet}
      isLoading={isLoading}
      onModalClose={() => navigate({ to: "/admin/policy_set" })}
      onEdit={(policyId: string) => {
        navigate({
          to: "/admin/policy_set/$policySetId/edit_policy/$policyId",
          params: {
            policyId: policyId,
            policySetId: params.policySetId,
          },
        });
      }}
      onAddPolicy={() => {
        navigate({
          to: "/admin/policy_set/$policySetId/add_policy",
          params: { policySetId: params.policySetId },
        });
      }}
    />
  );
}
