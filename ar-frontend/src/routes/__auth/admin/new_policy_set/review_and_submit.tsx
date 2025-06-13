import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCreatePolicySetContext } from "@/components/create-policy-set-context";
import { useAdminCreatePolicySet } from "@/network/policy-set";
import { ReviewAndSubmitStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/review_and_submit",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const { value } = useCreatePolicySetContext();
  const {
    mutateAsync: createPolicySet,
    isPending,
    error: submitError,
  } = useAdminCreatePolicySet();

  return (
    <ReviewAndSubmitStep
      onNext={() =>
        createPolicySet(value).then(() => navigate({ to: "/admin/policy_set" }))
      }
      nextPending={isPending}
      onBack={() => navigate({ to: "/admin/new_policy_set/add_policies" })}
      error={submitError?.message}
    />
  );
}
