import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCreatePolicySetContext } from "@/components/create-policy-set-context";
import { useCreatePolicySet } from "@/network/policy-set";
import { ReviewAndSubmitStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/member/new_policy_set/review_and_submit",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const {
    mutateAsync: createPolicySet,
    isPending,
    error: submitError,
  } = useCreatePolicySet();
  const { value } = useCreatePolicySetContext();

  function onBack() {
    navigate({
      to: "/member/new_policy_set/add_policies",
    });
  }

  function onNext() {
    createPolicySet(value).then(() => {
      navigate({
        to: "/member",
      });
    });
  }

  return (
    <ReviewAndSubmitStep
      onNext={onNext}
      nextPending={isPending}
      onBack={onBack}
      error={submitError?.message}
    />
  );
}
