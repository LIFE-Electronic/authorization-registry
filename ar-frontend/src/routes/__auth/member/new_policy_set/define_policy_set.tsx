import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { DefinePolicySetStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/member/new_policy_set/define_policy_set",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();

  return (
    <DefinePolicySetStep
      onBack={() => navigate({ to: "/member/new_policy_set/prefill_template" })}
      onNextNavigation={() =>
        navigate({ to: "/member/new_policy_set/add_policies" })
      }
    />
  );
}
