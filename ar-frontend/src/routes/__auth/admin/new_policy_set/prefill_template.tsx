import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { PrefillTemplateStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/prefill_template",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();

  return (
    <PrefillTemplateStep
      onBack={() => navigate({ to: "/admin/policy_set" })}
      onNextNavigation={() =>
        navigate({ to: "/admin/new_policy_set/define_policy_set" })
      }
    />
  );
}
