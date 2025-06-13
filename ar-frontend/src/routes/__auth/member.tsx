import {
  createFileRoute,
  Link,
  Outlet,
  useLocation,
  useNavigate,
} from "@tanstack/react-router";
import { Header, HeaderLink } from "@/components/header";
import { Box, Stack } from "@mui/joy";
import { PolicySetOverviewHeader } from "@/components/policy-set-overview";
import { usePolicySets } from "@/network/policy-set";
import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { PolicySetCard } from "@/components/policy-set-list-page";

export const Route = createFileRoute("/__auth/member")({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const location = useLocation();
  const { data: policySets, isLoading } = usePolicySets();

  return (
    <>
      <Outlet />
      <Box>
        <Header>
          <HeaderLink
            onClick={() => navigate({ to: "/member" })}
            selected={location.pathname.split("/")?.[1] === "member"}
          >
            Policy sets
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
              <PolicySetOverviewHeader
                onNewPolicySet={() =>
                  navigate({ to: "/member/new_policy_set/prefill_template" })
                }
              />
            </Box>
            <PageLoadingFallback isLoading={isLoading}>
              <Stack spacing={1} paddingY={2}>
                {policySets?.map((ps) => (
                  <Link
                    key={ps.policy_set_id}
                    style={{
                      textDecorationLine: "none",
                    }}
                    to="/member/policy_set/$policySetId"
                    params={{
                      policySetId: ps.policy_set_id,
                    }}
                  >
                    <PolicySetCard policySet={ps} />
                  </Link>
                ))}
              </Stack>
            </PageLoadingFallback>
          </Box>
        </Box>
      </Box>
    </>
  );
}
