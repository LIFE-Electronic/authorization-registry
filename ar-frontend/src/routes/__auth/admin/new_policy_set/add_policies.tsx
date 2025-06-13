import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AddPoliciesStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/add_policies",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();

  return (
    <AddPoliciesStep
      onBack={() => navigate({ to: "/admin/new_policy_set/define_policy_set" })}
      onNextNavigation={() =>
        navigate({ to: "/admin/new_policy_set/review_and_submit" })
      }
    />
  );
}
