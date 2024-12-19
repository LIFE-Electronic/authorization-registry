import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { AddEditPolicyContext } from "@/components/add-edit-policy-context";
import { Button, Stack, Typography, Box } from "@mui/joy";

export const Route = createFileRoute("/__auth/admin/new_policy_set/add_policy")(
  {
    component: Component,
  },
);

function Component() {
  const navigate = useNavigate();

  return (
    <AddEditPolicyContext>
      <Stack paddingY={2} spacing={2} direction="column">
        <Box>
          <Button
            onClick={() =>
              navigate({
                to: "/admin/new_policy_set/add_policies",
              })
            }
            variant="soft"
          >
            Back to policy set creation
          </Button>
        </Box>

        <Typography level="h3">Add policy to policy set</Typography>
      </Stack>
      <Outlet />
    </AddEditPolicyContext>
  );
}
