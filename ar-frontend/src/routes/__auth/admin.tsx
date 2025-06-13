import {
  createFileRoute,
  Outlet,
  useLocation,
  useNavigate,
} from "@tanstack/react-router";
import { CatchBoundary } from "@/components/catch-boundary";
import { Box } from "@mui/joy";
import { Header, HeaderLink } from "@/components/header";

export const Route = createFileRoute("/__auth/admin")({
  component: Component,
  errorComponent: CatchBoundary,
});

function Component() {
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <>
      <Header>
        <HeaderLink
          onClick={() => navigate({ to: "/admin/policy_set" })}
          selected={location.pathname === "/admin/policy_set"}
        >
          Policy sets
        </HeaderLink>
        <HeaderLink
          onClick={() => navigate({ to: "/admin/policy_set_templates" })}
          selected={location.pathname === "/admin/policy_set_templates"}
        >
          Policy set templates
        </HeaderLink>
      </Header>
      <Box
        width="100%"
        display="flex"
        alignItems="center"
        flexDirection="column"
      >
        <Box maxWidth={1360} width="100%" paddingX={4} boxSizing="border-box">
          <Box width="100%">
            <Outlet />
          </Box>
        </Box>
      </Box>
    </>
  );
}
