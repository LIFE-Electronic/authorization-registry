import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { CatchBoundary } from "@/components/catch-boundary";
import { DefinePolicySetStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/define_policy_set",
)({
  component: Component,
  errorComponent: CatchBoundary,
});

function Component() {
  const navigate = useNavigate();

  return (
    <DefinePolicySetStep
      onBack={() => navigate({ to: "/admin/new_policy_set/prefill_template" })}
      onNextNavigation={() =>
        navigate({ to: "/admin/new_policy_set/add_policies" })
      }
    />
  );
}
