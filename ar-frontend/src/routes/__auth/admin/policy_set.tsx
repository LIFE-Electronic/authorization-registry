import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { PolicySetCard } from "@/components/policy-set-list-page";
import { PolicySetOverviewHeader } from "@/components/policy-set-overview";
import { useAdminPolicySets } from "@/network/policy-set";
import { Box, Button, FormLabel, IconButton, Input, Stack } from "@mui/joy";
import ArrowBackIosIcon from "@mui/icons-material/ArrowBackIos";
import ArrowForwardIosIcon from "@mui/icons-material/ArrowForwardIos";
import {
  createFileRoute,
  Link,
  Outlet,
  useNavigate,
} from "@tanstack/react-router";
import { useDebounce } from "@uidotdev/usehooks";
import { useState } from "react";
import usePagination from "@mui/material/usePagination";

const ITEMS_ON_PAGE = 3;

function Pagination({
  numberOfItems,
  page,
  onPageChange,
}: {
  page: number;
  numberOfItems: number;
  onPageChange: (page: number) => void;
}) {
  const numberOfPages = Math.ceil(numberOfItems / ITEMS_ON_PAGE);
  const { items } = usePagination({
    count: numberOfPages,
    page,
    siblingCount: 2,
  });

  return (
    <nav>
      <Box display="flex" alignItems="center">
        {items.map((item, index) => {
          if (item.type === "start-ellipsis" || item.type === "end-ellipsis") {
            return <span key={index}>…</span>;
          }

          if (item.type === "previous") {
            return (
              <IconButton
                size="sm"
                onClick={() => onPageChange(page - 1)}
                key={index}
                disabled={item.disabled}
              >
                <ArrowBackIosIcon />
              </IconButton>
            );
          }

          if (item.type === "next") {
            return (
              <IconButton
                size="sm"
                onClick={() => onPageChange(page + 1)}
                key={index}
                disabled={item.disabled}
              >
                <ArrowForwardIosIcon />
              </IconButton>
            );
          }

          if (item.type === "page") {
            return (
              <Button
                onClick={() => item.page !== null && onPageChange(item.page)}
                key={index}
                variant={item.selected ? "solid" : "plain"}
                color="neutral"
                sx={
                  item.selected
                    ? {
                        backgroundColor: "#363D44",
                      }
                    : {}
                }
              >
                {item.page}
              </Button>
            );
          }
        })}
      </Box>
    </nav>
  );
}

export const Route = createFileRoute("/__auth/admin/policy_set")({
  component: Component,
});

function Component() {
  const [page, setPage] = useState(1);
  const [accessSubject, setAccessSubject] = useState<string | undefined>(
    undefined,
  );
  const [policyIssuer, setPolicyIssuer] = useState<string | undefined>(
    undefined,
  );

  const deboundedAccessSubject = useDebounce(accessSubject, 300);
  const debouncedPolicyIssuer = useDebounce(policyIssuer, 300);

  const navigate = useNavigate();
  const { data: policySets, isLoading } = useAdminPolicySets({
    accessSubject: deboundedAccessSubject,
    policyIssuer: debouncedPolicyIssuer,
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
          <FormLabel>Policy issuer</FormLabel>
          <Input
            size="sm"
            value={policyIssuer}
            onChange={(e) => {
              setPolicyIssuer(e.target.value);
              setPage(1);
            }}
          />
        </Box>
        <Box sx={{ width: 180 }}>
          <FormLabel>Access subject</FormLabel>
          <Input
            size="sm"
            value={accessSubject}
            onChange={(e) => {
              setAccessSubject(e.target.value);
              setPage(1);
            }}
          />
        </Box>
      </Stack>
      <PageLoadingFallback isLoading={isLoading}>
        <Stack spacing={1} paddingBottom={2}>
          <Box display="flex" paddingY={2}>
            <Pagination
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
