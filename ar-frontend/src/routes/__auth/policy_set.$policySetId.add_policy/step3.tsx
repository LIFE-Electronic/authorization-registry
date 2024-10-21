import { Button, Stack } from "@mui/joy";
import { createFileRoute } from "@tanstack/react-router";
import { AddPolicyStepper } from "../../../components/add-policy-stepper";
import { z } from "zod";
import { PolicyCard } from "../../../components/policy-card";
import { useAddPolicyToPolicySet } from "../../../network/policy-set";

const searchSchema = z.object({
  actions: z.array(z.string()),
  resource_type: z.string(),
  identifiers: z.array(z.string()),
  attributes: z.array(z.string()),
  service_providers: z.array(z.string()),
  rules: z.array(
    z.object({
      resource_type: z.string(),
      identifiers: z.array(z.string()),
      attributes: z.array(z.string()),
      actions: z.array(z.string()),
    }),
  ),
});

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step3",
)({
  component: Component,
  validateSearch: searchSchema,
});

function Component() {
  const params = Route.useParams();
  const search = Route.useSearch();
  const denyRules = search.rules.map((r) => ({
    target: {
      actions: r.actions,
      resource: {
        attributes: r.attributes,
        identifiers: r.identifiers,
        type: r.resource_type,
      },
    },
    effect: "Deny" as const,
  }));

  const { mutateAsync: addPolicy } = useAddPolicyToPolicySet({
    policySetId: params.policySetId,
  });

  return (
    <Stack spacing={3}>
      <AddPolicyStepper activeStep={3} />
      <PolicyCard policy={{ ...search, rules: denyRules }} />

      <Stack direction="row">
        <Button
          onClick={() => {
            addPolicy({
              policy: {
                ...search,
                rules: denyRules,
              },
            });
          }}
        >
          Add policy
        </Button>
      </Stack>
    </Stack>
  );
}
