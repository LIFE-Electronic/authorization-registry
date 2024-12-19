import { Button, Input, Stack } from "@mui/joy";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AddPolicySetStepper } from "@/components/add-policy-set-stepper";
import { useCreatePolicySetContext } from "@/components/create-policy-set-context";
import { useForm } from "@tanstack/react-form";
import { FormField } from "@/components/form-field";
import { required } from "@/form-field-validators";

export const Route = createFileRoute(
  "/__auth/member/new_policy_set/define_policy_set",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const { value, changeValue } = useCreatePolicySetContext();

  const form = useForm({
    defaultValues: {
      access_subject: value.access_subject,
      policy_issuer: value.policy_issuer,
    },
    onSubmit: ({ value }) => {
      changeValue((oldValue) => ({ ...oldValue, ...value }));
      navigate({ to: "/member/new_policy_set/add_policies" });
    },
  });

  return (
    <div>
      <AddPolicySetStepper activeStep="Define policy set" />
      <form
        onSubmit={(e) => {
          e.preventDefault();
          e.stopPropagation();
          form.handleSubmit();
        }}
      >
        <Stack paddingTop={2} spacing={1}>
          <form.Field
            name="access_subject"
            validators={required}
            children={(field) => (
              <FormField
                label="Access subject"
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
            name="policy_issuer"
            validators={required}
            children={(field) => (
              <FormField label="Policy issuer" errors={field.state.meta.errors}>
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
              </FormField>
            )}
          />
          <Stack direction="row" spacing={1}>
            <Button
              variant="outlined"
              onClick={() =>
                navigate({ to: "/member/new_policy_set/prefill_template" })
              }
            >
              Back
            </Button>
            <Button type="submit">Next step</Button>
          </Stack>
        </Stack>
      </form>
    </div>
  );
}
