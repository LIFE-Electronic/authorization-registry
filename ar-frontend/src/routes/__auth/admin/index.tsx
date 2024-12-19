import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useAdminPolicySets } from "@/network/policy-set";
import { Box, Button, FormLabel, Input, Stack, Typography } from "@mui/joy";
import { z } from "zod";
import { useDebounce } from "@uidotdev/usehooks";
import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { CatchBoundary } from "@/components/catch-boundary";
import { PolicySetCard } from "@/components/policy-set-list-page";

const searchSchema = z.object({
  access_subject: z.string().optional(),
  policy_issuer: z.string().optional(),
});

export const Route = createFileRoute("/__auth/admin/")({
  component: Component,
  validateSearch: searchSchema,
  errorComponent: CatchBoundary,
});

function Component() {
  const search = Route.useSearch();
  const accessSubject = useDebounce(search.access_subject, 300);
  const policyIssuer = useDebounce(search.policy_issuer, 300);
  const navigate = useNavigate();
  const { data: policySets, isLoading } = useAdminPolicySets({
    accessSubject,
    policyIssuer,
  });

  return (
    <div>
      <Typography level="h2">Policy sets</Typography>
      <Stack paddingY={2} spacing={2} direction="row" alignItems="flex-end">
        <Box sx={{ width: 180 }}>
          <FormLabel>Access subject</FormLabel>
          <Input
            size="sm"
            defaultValue={search.access_subject || ""}
            onChange={(e) =>
              navigate({
                to: "/admin",
                search: {
                  ...search,
                  access_subject: e.target.value,
                },
              })
            }
          />
        </Box>
        <Box sx={{ width: 180 }}>
          <FormLabel>Policy issuer</FormLabel>
          <Input
            size="sm"
            defaultValue={search.policy_issuer || ""}
            onChange={(e) =>
              navigate({
                to: "/admin",
                search: {
                  ...search,
                  policy_issuer: e.target.value,
                },
              })
            }
          />
        </Box>
        <Box>
          <Button
            onClick={() =>
              navigate({ to: "/admin/new_policy_set/prefill_template" })
            }
          >
            New policy set
          </Button>
        </Box>
      </Stack>
      <PageLoadingFallback isLoading={isLoading}>
        <Stack spacing={1}>
          {policySets?.map((ps) => (
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
    </div>
  );
}
