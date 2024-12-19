import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { usePolicySets } from "@/network/policy-set";
import { Box, Button, Stack, Typography } from "@mui/joy";
import { z } from "zod";
import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { CatchBoundary } from "@/components/catch-boundary";
import { PolicySetCard } from "@/components/policy-set-list-page";

const searchSchema = z.object({
  access_subject: z.string().optional(),
  policy_issuer: z.string().optional(),
});

export const Route = createFileRoute("/__auth/member/")({
  component: Component,
  validateSearch: searchSchema,
  errorComponent: CatchBoundary,
});

function Component() {
  const navigate = useNavigate();
  const { data: policySets, isLoading } = usePolicySets();

  return (
    <div>
      <Stack
        paddingBottom={2}
        direction="row"
        justifyContent="space-between"
        spacing={1}
      >
        <Typography level="h2">Policy sets</Typography>
        <Box>
          <Button
            variant="plain"
            onClick={() =>
              navigate({ to: "/member/new_policy_set/prefill_template" })
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
    </div>
  );
}
