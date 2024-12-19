import { createFileRoute, Outlet } from "@tanstack/react-router";
import { Typography } from "@mui/joy";
import { CreatePolicySetContext } from "@/components/create-policy-set-context";

export const Route = createFileRoute("/__auth/admin/new_policy_set")({
  component: Component,
});

function Component() {
  return (
    <CreatePolicySetContext>
      <Typography paddingBottom={2} level="h2">
        New policy set
      </Typography>

      <Outlet />
    </CreatePolicySetContext>
  );
}
