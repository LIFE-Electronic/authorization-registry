import { Stack, Box, Alert, FormLabel, FormHelperText, Input } from "@mui/joy";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCreatePolicySetTemplateContext } from "../policy_set_templates.new_policy_set_template";
import { PolicyCard } from "@/components/policy-card";
import { useAdminCreatePolicySetTemplate } from "@/network/policy-set-templates";
import { NewPolicySetTemplateModalWrapper } from "@/components/new-policy-set-template";
import { FormField } from "@/components/form-field";

export const Route = createFileRoute(
  "/__auth/admin/policy_set_templates/new_policy_set_template/review_and_submit",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const { value } = useCreatePolicySetTemplateContext();
  const {
    mutateAsync: createPolicySetTemplate,
    isPending,
    error,
  } = useAdminCreatePolicySetTemplate();

  function onSubmit() {
    createPolicySetTemplate(value).then(() => {
      navigate({
        to: "/admin/policy_set_templates",
      });
    });
  }

  return (
    <NewPolicySetTemplateModalWrapper
      step="Review and submit"
      nextDisabled={isPending}
      onNext={onSubmit}
      nextText="Save policy set template"
      onBack={() =>
        navigate({
          to: "/admin/policy_set_templates/new_policy_set_template/add_policies",
        })
      }
    >
      <Stack width="100%" spacing={2}>
        <Box>
          <FormLabel sx={{ fontSize: "16px" }}>Review and submit</FormLabel>
          <FormHelperText sx={{ fontSize: "16px" }}>
            View the overview, create the policy set template or go back and
            make some modifications
          </FormHelperText>
        </Box>
        {error && (
          <Box>
            <Alert color="danger">
              <Box>{error.message}</Box>
            </Alert>
          </Box>
        )}
        <FormField label="Name" errors={[]}>
          <Input
            sx={(theme) => ({
              "&.Mui-disabled": {
                backgroundColor: "#F4F5F6",
                color: "#212529",
                borderColor: theme.vars.palette.neutral[200],
              },
            })}
            disabled
            value={value.name}
          />
        </FormField>
        <FormField label="Description" errors={[]}>
          <Input
            sx={(theme) => ({
              "&.Mui-disabled": {
                backgroundColor: "#F4F5F6",
                color: "#212529",
                borderColor: theme.vars.palette.neutral[200],
              },
            })}
            disabled
            value={value.description}
          />
        </FormField>
        <FormField label="Policy issuer" errors={[]}>
          <Input
            sx={(theme) => ({
              "&.Mui-disabled": {
                backgroundColor: "#F4F5F6",
                color: "#212529",
                borderColor: theme.vars.palette.neutral[200],
              },
            })}
            disabled
            value={value.policy_issuer}
          />
        </FormField>

        <FormField label="Access subject" errors={[]}>
          <Input
            sx={(theme) => ({
              "&.Mui-disabled": {
                backgroundColor: "#F4F5F6",
                color: "#212529",
                borderColor: theme.vars.palette.neutral[200],
              },
            })}
            disabled
            value={value.access_subject}
          />
        </FormField>

        <Box>
          <FormLabel>Policies</FormLabel>
          <Box flexWrap="wrap" display="flex" gap={1} paddingBottom={2}>
            {value.policies.map((p, idx) => (
              <Box
                key={idx}
                width={{
                  xs: "100%",
                  sm: "47%",
                  md: "32%",
                }}
                height="100%"
              >
                <PolicyCard detailed policy={p} />
              </Box>
            ))}
          </Box>
        </Box>
      </Stack>
    </NewPolicySetTemplateModalWrapper>
  );
}
