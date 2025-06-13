import { createFileRoute, Outlet } from "@tanstack/react-router";
import { CreatePolicySetContext } from "@/components/create-policy-set-context";

export const Route = createFileRoute("/__auth/member/new_policy_set")({
  component: Component,
});

function Component() {
  return (
    <CreatePolicySetContext>
      <Outlet />
    </CreatePolicySetContext>
  );
}
