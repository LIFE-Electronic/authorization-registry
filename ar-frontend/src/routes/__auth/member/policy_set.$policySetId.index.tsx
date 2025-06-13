import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  usePolicySet,
  useDeletePolicySet,
  useDeletePolicyFromPolicySet,
} from "@/network/policy-set";
import { CatchBoundary } from "@/components/catch-boundary";
import { PolicySetDetail } from "@/components/policy-set-detail-page";

export const Route = createFileRoute("/__auth/member/policy_set/$policySetId/")(
  {
    component: Component,
    errorComponent: CatchBoundary,
  },
);

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();
  const { data: policySet, isLoading } = usePolicySet({
    policySetId: params.policySetId,
  });
  const {
    mutateAsync: deletePolicySet,
    isPending: isDeletePending,
    error: deleteError,
  } = useDeletePolicySet({
    policySetId: params.policySetId,
  });

  function onDeletePolicySet() {
    deletePolicySet().then(() => {
      navigate({
        replace: true,
        to: "/member",
      });
    });
  }

  const {
    mutateAsync: deletePolicy,
    isPending: isDeletePolicyPending,
    error: deletePolicyError,
  } = useDeletePolicyFromPolicySet({
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
      onModalClose={() => navigate({ to: "/member" })}
      onEdit={(policyId: string) => {
        navigate({
          to: "/member/policy_set/$policySetId/edit_policy/$policyId",
          params: {
            policyId: policyId,
            policySetId: params.policySetId,
          },
        });
      }}
      onAddPolicy={() => {
        navigate({
          to: "/member/policy_set/$policySetId/add_policy",
          params: { policySetId: params.policySetId },
        });
      }}
    />
  );
}
