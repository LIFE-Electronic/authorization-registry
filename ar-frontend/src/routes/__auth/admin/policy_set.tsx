import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { PolicySetCard } from "@/components/policy-set-list-page";
import { PolicySetOverviewHeader } from "@/components/policy-set-overview";
import { useAdminPolicySets } from "@/network/policy-set";
import { Box, FormLabel, Input, Stack } from "@mui/joy";
import {
  createFileRoute,
  Link,
  Outlet,
  useNavigate,
} from "@tanstack/react-router";
import { useDebounce } from "@uidotdev/usehooks";
import { useState } from "react";
import { Pagination } from "@/components/pagination";

const ITEMS_ON_PAGE = 3;

export const Route = createFileRoute("/__auth/admin/policy_set")({
  component: Component,
});

function Component() {
  const [page, setPage] = useState(1);
  const [q, setQ] = useState<string>("");
  const deboundedQ = useDebounce(q, 300);

  const navigate = useNavigate();
  const { data: policySets, isLoading } = useAdminPolicySets({
    q: deboundedQ,
    skip: ITEMS_ON_PAGE * (page - 1),
    limit: ITEMS_ON_PAGE,
  });

  return (
    <>
      <Outlet />
      <PolicySetOverviewHeader
        onNewPolicySet={() =>
          navigate({ to: "/admin/new_policy_set/prefill_template" })
        }
      />
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
        <Stack spacing={1} paddingBottom={2}>
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
              to="/admin/policy_set/$policySetId"
              params={{
                policySetId: ps.policy_set_id,
              }}
            >
              <PolicySetCard policySet={ps} />
            </Link>
          ))}
        </Stack>
      </PageLoadingFallback>
    </>
  );
}
