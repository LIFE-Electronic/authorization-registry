import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useAdminPolicySet } from "../../network/policy-set";
import { PageLoadingFallback } from "../../components/page-loading-fallback";
import { CatchBoundary } from "../../components/catch-boundary";
import {
  Box,
  Typography,
  Stack,
  Button,
} from "@mui/joy";
import { PolicyCard } from "../../components/policy-card";
import { z } from "zod";

const searchSchema = z.object({
  add_policy: z.boolean().optional(),
});

export const Route = createFileRoute("/__auth/policy_set/$policySetId/")({
  component: Component,
  errorComponent: CatchBoundary,
  validateSearch: searchSchema,
});

function Component() {
  const navigate = useNavigate();
  const { policySetId } = Route.useParams();

  const { data: policySet, isLoading } = useAdminPolicySet({
    policySetId,
  });

  return (
    <PageLoadingFallback isLoading={isLoading}>
      {policySet && (
        <Stack spacing={3}>
          <Box>
            <Typography level="title-md">Access subject</Typography>
            <Typography>{policySet.access_subject}</Typography>
          </Box>
          <Box>
            <Typography level="title-md">Policy issuer</Typography>
            <Typography>{policySet.policy_issuer}</Typography>
          </Box>
          <Box>
            <Typography>Policies</Typography>
            <Stack spacing={1} direction="row">
              {policySet.policies.map((p) => (
                <PolicyCard policy={p} key={p.id} />
              ))}
            </Stack>
            <Box paddingTop={2}>
              <Button
                onClick={() =>
                  navigate({
                    to: "/policy_set/$policySetId/add_policy/step1",
                    params: { policySetId },
                  })
                }
              >
                Add policy
              </Button>
            </Box>
          </Box>
        </Stack>
      )}
    </PageLoadingFallback>
  );
}
