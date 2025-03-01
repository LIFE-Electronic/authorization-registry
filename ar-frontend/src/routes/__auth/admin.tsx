import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { initLogout } from "@/network/idp";
import { Box, Button } from "@mui/joy";
import { Logo } from "@/components/logo";

export const Route = createFileRoute("/__auth/admin")({
  component: Component,
});

function Component() {
  const navigate = useNavigate();

  return (
    <Box>
      <Box
        display="flex"
        alignItems="center"
        justifyContent="space-between"
        paddingY={2}
      >
        <Logo admin />
        <Box>
          <Button
            variant="plain"
            color="neutral"
            onClick={() => navigate({ to: "/admin" })}
          >
            Policy sets
          </Button>
          <Button
            variant="plain"
            color="neutral"
            onClick={() => navigate({ to: "/admin/policy_set_templates" })}
          >
            Policy set templates
          </Button>
        </Box>

        <Button variant="soft" onClick={() => initLogout()}>
          Logout
        </Button>
      </Box>
      <Outlet />
    </Box>
  );
}
