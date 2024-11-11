import { createFileRoute, Outlet } from "@tanstack/react-router";
import { AddPolicyContext } from "../../components/add-policy-context";

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy",
)({
  component: Component,
});

function Component() {
  return (
    <AddPolicyContext>
      <Outlet />
    </AddPolicyContext>
  );
}
