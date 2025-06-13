import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { PrefillTemplateStep } from "@/components/new-policy-set";

export const Route = createFileRoute(
  "/__auth/member/new_policy_set/prefill_template",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();

  return (
    <PrefillTemplateStep
      onBack={() => navigate({ to: "/member" })}
      onNextNavigation={() =>
        navigate({ to: "/member/new_policy_set/define_policy_set" })
      }
    />
  );
}
