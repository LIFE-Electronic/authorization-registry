import { Stack, Typography, Box, Button, Divider, Alert } from "@mui/joy";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCreatePolicySetContext } from "@/components/create-policy-set-context";
import { PolicyCard } from "@/components/policy-card";
import { useAdminCreatePolicySet } from "@/network/policy-set";
import { AddPolicySetStepper } from "@/components/add-policy-set-stepper";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/review_and_submit",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const {
    mutateAsync: createPolicySet,
    isPending,
    error: submitError,
  } = useAdminCreatePolicySet();
  const { value } = useCreatePolicySetContext();

  function onBack() {
    navigate({
      to: "/admin/new_policy_set/add_policies",
    });
  }

  return (
    <Stack spacing={3}>
      <AddPolicySetStepper activeStep="Review and submit" />

      {submitError && (
        <Box paddingTop={4}>
          <Alert color="danger">
            <Box>{submitError.message}</Box>
          </Alert>
        </Box>
      )}

      <Box>
        <Typography level="title-lg">Policy issuer</Typography>
        <Typography>{value.policy_issuer}</Typography>
      </Box>

      <Box>
        <Typography level="title-lg">Access subject</Typography>
        <Typography>{value.access_subject}</Typography>
      </Box>

      <Box>
        <Typography level="title-lg">Policies</Typography>
        <Stack direction="row" spacing={1}>
          {value.policies.map((p, idx) => (
            <PolicyCard policy={p} key={idx} />
          ))}
        </Stack>
      </Box>

      <Divider />

      <Stack direction="row" spacing={1}>
        <Button onClick={onBack} variant="outlined">
          Back
        </Button>
        <Button
          disabled={isPending}
          onClick={() =>
            createPolicySet(value).then(() => {
              navigate({
                to: "/admin",
              });
            })
          }
        >
          Submit
        </Button>
      </Stack>
    </Stack>
  );
}
