import { Button, Stack } from "@mui/joy";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AddPolicyStepper } from "../../../components/add-policy-stepper";
import { PolicyCard } from "../../../components/policy-card";
import { useAddPolicyToPolicySet } from "../../../network/policy-set";
import { useAddPolicyContext } from "../policy_set.$policySetId.add_policy";

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step3",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();
  const { value } = useAddPolicyContext();
  const policy = {
    ...value,
    rules: [{ effect: "Permit" as const }, ...value.rules],
  };

  const { mutateAsync: addPolicy, isPending } = useAddPolicyToPolicySet({
    policySetId: params.policySetId,
  });

  return (
    <Stack spacing={3}>
      <AddPolicyStepper activeStep={3} />
      <PolicyCard policy={policy} />

      <Stack direction="row" spacing={1}>
        <Button
          variant="outlined"
          onClick={() => {
            navigate({
              to: "/policy_set/$policySetId/add_policy/step2",
              params,
            });
          }}
        >
          Back
        </Button>
        <Button
          disabled={isPending}
          onClick={() => {
            addPolicy({
              policy,
            }).then(() => {
              navigate({ to: "/policy_set/$policySetId", params });
            });
          }}
        >
          Add policy
        </Button>
      </Stack>
    </Stack>
  );
}
