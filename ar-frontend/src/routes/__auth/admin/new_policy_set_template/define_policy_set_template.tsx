import { AddPolicySetTemplateStepper } from "@/components/wizzard-stepper";
import { useCreatePolicySetTemplateContext } from "../new_policy_set_template";
import { FormField } from "@/components/form-field";
import { Box, Button, FormHelperText, Input, Stack } from "@mui/joy";
import { useForm } from "@tanstack/react-form";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { required } from "@/form-field-validators";

export const Route = createFileRoute(
  "/__auth/admin/new_policy_set_template/define_policy_set_template",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const { value, changeValue } = useCreatePolicySetTemplateContext();

  const form = useForm({
    defaultValues: {
      name: value.name,
      access_subject: value.access_subject,
      policy_issuer: value.policy_issuer,
    },
    onSubmit: ({ value }) => {
      changeValue((oldValue) => ({ ...oldValue, ...value }));

      // have to validate here
      navigate({ to: "/admin/new_policy_set_template/add_policies" });
    },
  });

  return (
    <Box paddingTop={2}>
      <AddPolicySetTemplateStepper activeStep="Define policy set template" />
      <form
        onSubmit={(e) => {
          e.preventDefault();
          e.stopPropagation();
          form.handleSubmit();
        }}
      >
        <Stack paddingTop={2} spacing={1}>
          <form.Field
            name="name"
            validators={required}
            children={(field) => (
              <FormField
                label="Policy template name"
                errors={field.state.meta.errors}
              >
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
              </FormField>
            )}
          />
          <form.Field
            name="access_subject"
            children={(field) => (
              <FormField
                label="Access subject"
                errors={field.state.meta.errors}
              >
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
                <FormHelperText>
                  Leave this value empty if you don't want to prefill the access
                  subject
                </FormHelperText>
              </FormField>
            )}
          />
          <form.Field
            name="policy_issuer"
            children={(field) => (
              <FormField label="Policy issuer" errors={field.state.meta.errors}>
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
                <FormHelperText>
                  Leave this value empty if you don't want to prefill the policy
                  issuer
                </FormHelperText>
              </FormField>
            )}
          />
          <Stack direction="row" spacing={1}>
            <Button type="submit">Next step</Button>
          </Stack>
        </Stack>
      </form>
    </Box>
  );
}
