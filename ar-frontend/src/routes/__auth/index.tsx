import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import {
  PolicySetWithPolicies,
  useAdminPolicySets,
} from "../../network/policy-set";
import {
  Box,
  Button,
  Card,
  FormLabel,
  Input,
  Stack,
  Typography,
} from "@mui/joy";
import { z } from "zod";
import { useDebounce } from "@uidotdev/usehooks";
import { PageLoadingFallback } from "../../components/page-loading-fallback";
import { CatchBoundary } from "../../components/catch-boundary";
import { PolicyCard } from "../../components/policy-card";

const searchSchema = z.object({
  access_subject: z.string().optional(),
  policy_issuer: z.string().optional(),
});

export const Route = createFileRoute("/__auth/")({
  component: Component,
  validateSearch: searchSchema,
  errorComponent: CatchBoundary,
});

function PolicySetCard({ policySet }: { policySet: PolicySetWithPolicies }) {
  return (
    <Card>
      <Stack direction="row" spacing={2}>
        <Box>
          <Typography level="title-sm">Access subject</Typography>
          <Typography level="body-xs">{policySet.access_subject}</Typography>
        </Box>
        <Box>
          <Typography level="title-sm">Policy issuer</Typography>
          <Typography level="body-xs">{policySet.policy_issuer}</Typography>
        </Box>
      </Stack>
      <Box>
        <Typography>Policies</Typography>
        <Stack spacing={2} direction="row" flexWrap="wrap" useFlexGap>
          {policySet.policies.map((p) => (
            <Box
              key={p.id}
              width={{
                xs: "100%",
                sm: "47%",
                md: "32%",
              }}
              height="100%"
            >
              <PolicyCard policy={p} />
            </Box>
          ))}
        </Stack>
      </Box>
    </Card>
  );
}

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
                to: "/",
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
                to: "/",
                search: {
                  ...search,
                  policy_issuer: e.target.value,
                },
              })
            }
          />
        </Box>
        <Box>
          <Button onClick={() => navigate({ to: "/new_policy_set/step1" })}>
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
              to="/policy_set/$policySetId"
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
