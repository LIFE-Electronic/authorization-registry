import { AddPolicySetStepper } from "@/components/add-policy-set-stepper";
import { useCreatePolicySetContext } from "@/components/create-policy-set-context";
import { policySetTemplates } from "@/policy-set-templates";
import { Button, Stack, Box, Select, Option, FormLabel } from "@mui/joy";
import { createFileRoute, redirect, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set/prefill_template",
)({
  component: Component,
  loader: () => {
    if (policySetTemplates.length === 0) {
      throw redirect({ to: "/admin/new_policy_set/define_policy_set" });
    }
  },
});

function Component() {
  const { changeValue } = useCreatePolicySetContext();
  const navigate = useNavigate();
  const [template, setTemplate] = useState<number>();

  function applyTemplate() {
    if (template !== undefined) {
      const policySetTemplate = policySetTemplates[template];

      changeValue({
        access_subject: policySetTemplate.access_subject || "",
        policy_issuer: policySetTemplate.policy_issuer || "",
        policies: policySetTemplate.policies,
      });
    }
  }

  return (
    <Stack spacing={2}>
      <AddPolicySetStepper activeStep="Prefill from template" />

      <Box>
        <FormLabel>Policy set template</FormLabel>
        {/* @ts-expect-error joy-ui is not smart enough to infer the type from dynamic options */}
        <Select onChange={(_, newValue) => setTemplate(newValue)}>
          {policySetTemplates.map((ps, idx) => (
            <Option key={idx} value={idx} label={ps.name}>
              {ps.name}
            </Option>
          ))}
        </Select>
      </Box>

      <Stack direction="row" spacing={1}>
        <Button disabled={template === undefined} onClick={applyTemplate}>
          Apply template
        </Button>
        <Button
          variant="outlined"
          onClick={() =>
            navigate({ to: "/admin/new_policy_set/define_policy_set" })
          }
        >
          Next
        </Button>
      </Stack>
    </Stack>
  );
}
