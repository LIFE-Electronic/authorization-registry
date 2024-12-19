import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AddPolicySetStepper } from "@/components/add-policy-set-stepper";
import { Typography, Button, Stack, Box, Divider, Card } from "@mui/joy";
import { useCreatePolicySetContext } from "@/components/create-policy-set-context";
import { PolicyCard } from "@/components/policy-card";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/add_policies",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();

  const { value, changeValue } = useCreatePolicySetContext();

  return (
    <Stack spacing={3}>
      <AddPolicySetStepper activeStep="Add policies" />

      <Divider />

      <Typography level="title-lg">Policies</Typography>
      <Stack direction="row" spacing={1}>
        {value.policies.map((p, idx) => (
          <Card key={idx}>
            <PolicyCard policy={p} key={idx} />
            <Stack direction="row">
              <Button
                onClick={() =>
                  changeValue((value) => ({
                    ...value,
                    policies: value.policies.filter((_, pidx) => pidx !== idx),
                  }))
                }
                size="sm"
                variant="outlined"
                color="danger"
              >
                Delete
              </Button>
            </Stack>
          </Card>
        ))}
      </Stack>

      <Box>
        <Button
          onClick={() =>
            navigate({ to: "/admin/new_policy_set/add_policy/step1" })
          }
          variant="outlined"
        >
          Add policy
        </Button>
      </Box>

      <Divider />
      <Stack direction="row" spacing={1}>
        <Button
          variant="outlined"
          onClick={() =>
            navigate({ to: "/admin/new_policy_set/define_policy_set" })
          }
        >
          Back
        </Button>
        <Button
          onClick={() =>
            navigate({ to: "/admin/new_policy_set/review_and_submit" })
          }
        >
          Next
        </Button>
      </Stack>
    </Stack>
  );
}
