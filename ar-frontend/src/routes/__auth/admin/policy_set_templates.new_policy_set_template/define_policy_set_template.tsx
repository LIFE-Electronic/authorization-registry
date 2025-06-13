import { useCreatePolicySetTemplateContext } from "../policy_set_templates.new_policy_set_template";
import { FormField } from "@/components/form-field";
import { FormHelperText, Input, Stack } from "@mui/joy";
import { useForm } from "@tanstack/react-form";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { required } from "@/form-field-validators";
import { NewPolicySetTemplateModalWrapper } from "@/components/new-policy-set-template";

export const Route = createFileRoute(
  "/__auth/admin/policy_set_templates/new_policy_set_template/define_policy_set_template",
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
      description: value.description,
    },
    onSubmit: ({ value }) => {
      changeValue((oldValue) => ({ ...oldValue, ...value }));

      // have to validate here
      navigate({
        to: "/admin/policy_set_templates/new_policy_set_template/add_policies",
      });
    },
  });

  return (
    <NewPolicySetTemplateModalWrapper
      step="Define policy set template"
      onNext={() => {
        form.handleSubmit();
      }}
      onBack={() => {
        navigate({ to: "/admin/policy_set_templates" });
      }}
    >
      <form
        style={{
          width: "100%",
        }}
        onSubmit={(e) => {
          e.preventDefault();
          e.stopPropagation();
          form.handleSubmit();
        }}
      >
        <Stack width="100%" spacing={2}>
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
            name="description"
            children={(field) => (
              <FormField label="Description" errors={field.state.meta.errors}>
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
        </Stack>
      </form>
    </NewPolicySetTemplateModalWrapper>
  );
}
