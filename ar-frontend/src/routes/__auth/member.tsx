import {
  createFileRoute,
  Link,
  Outlet,
  useLocation,
  useNavigate,
} from "@tanstack/react-router";
import { Header, HeaderLink } from "@/components/header";
import { Box, FormLabel, Input, Stack } from "@mui/joy";
import { PolicySetOverviewHeader } from "@/components/policy-set-overview";
import { usePolicySets } from "@/network/policy-set";
import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { PolicySetCard } from "@/components/policy-set-list-page";
import { Pagination } from "@/components/pagination";
import { useDebounce } from "@uidotdev/usehooks";
import { useState } from "react";

const ITEMS_ON_PAGE = 3;

export const Route = createFileRoute("/__auth/member")({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const location = useLocation();

  const [page, setPage] = useState(1);
  const [q, setQ] = useState<string>("");
  const deboundedQ = useDebounce(q, 300);
  const { data: policySets, isLoading } = usePolicySets({
    q: deboundedQ,
    skip: ITEMS_ON_PAGE * (page - 1),
    limit: ITEMS_ON_PAGE,
  });

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
            <Stack
              paddingBottom={2}
              spacing={2}
              direction="row"
              alignItems="flex-end"
            >
              <Box sx={{ width: 180 }}>
                <FormLabel>Search</FormLabel>
                <Input
                  size="sm"
                  value={q}
                  onChange={(e) => {
                    setQ(e.target.value);
                    setPage(1);
                  }}
                />
              </Box>
            </Stack>
            <PageLoadingFallback isLoading={isLoading}>
              <Stack spacing={1} paddingY={2}>
                <Box display="flex" paddingY={2}>
                  <Pagination
                    itemsOnPage={ITEMS_ON_PAGE}
                    page={page}
                    onPageChange={(page) => setPage(page)}
                    numberOfItems={policySets?.pagination.total_count || 0}
                  />
                </Box>
                {policySets?.data.map((ps) => (
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
