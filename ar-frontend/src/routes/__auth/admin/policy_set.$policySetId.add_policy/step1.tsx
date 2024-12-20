import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Step1 } from "@/components/add-edit-policy";

export const Route = createFileRoute(
  "/__auth/admin/policy_set/$policySetId/add_policy/step1",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();
  function onSubmit() {
    navigate({
      to: "/admin/policy_set/$policySetId/add_policy/step2",
      params,
    });
  }

  return <Step1 onSubmit={onSubmit} />;
}
