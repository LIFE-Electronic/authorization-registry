import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useForm } from "@tanstack/react-form";
import {
  Stack,
  Box,
  Button,
  Select,
  Option,
  Input,
  Autocomplete,
  FormHelperText,
} from "@mui/joy";
import { AddPolicyStepper } from "../../../components/add-policy-stepper";
import { required } from "../../../form-field-validators";
import { FormField } from "../../../components/form-field";
import { useAddPolicyContext } from "../policy_set.$policySetId.add_policy";

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step1",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();
  const { value, changeValue } = useAddPolicyContext();

  const form = useForm<{
    actions: string[];
    resource_type: string;
    identifiers: string[];
    attributes: string[];
    service_providers: string[];
  }>({
    defaultValues: value,
    onSubmit: ({ value }) => {
      changeValue((oldValue) => ({ ...oldValue, ...value }));
      navigate({
        to: "/policy_set/$policySetId/add_policy/step2",
        search: { ...value, rules: [] },
        params,
      });
    },
  });

  return (
    <Stack spacing={3}>
      <AddPolicyStepper activeStep={1} />

      <form
        onSubmit={(e) => {
          e.preventDefault();
          e.stopPropagation();
          form.handleSubmit();
        }}
      >
        <Stack spacing={1}>
          <form.Field
            name="actions"
            validators={required}
            children={(field) => (
              <FormField label="Actions" errors={field.state.meta.errors}>
                <Select
                  value={field.state.value}
                  onChange={(_, newValue) => field.handleChange(newValue)}
                  multiple
                >
                  <Option value="read">Read</Option>
                  <Option value="edit">Edit</Option>
                  <Option value="delete">Delete</Option>
                </Select>
              </FormField>
            )}
          />
          <form.Field
            name="resource_type"
            validators={required}
            children={(field) => (
              <FormField label="Resource type" errors={field.state.meta.errors}>
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
              </FormField>
            )}
          />
          <form.Field
            name="service_providers"
            validators={required}
            children={(field) => (
              <FormField
                errors={field.state.meta.errors}
                label="Service providers"
              >
                <Autocomplete
                  clearOnBlur
                  value={field.state.value}
                  onChange={(_, value) => {
                    console.log("chnginnnng", { value });
                    field.handleChange(value);
                  }}
                  freeSolo
                  multiple
                  options={[]}
                  error={field.state.meta.errors.length > 0}
                />
              </FormField>
            )}
          />
          <form.Field
            name="identifiers"
            validators={required}
            children={(field) => (
              <FormField label="Identifiers" errors={field.state.meta.errors}>
                <Autocomplete
                  value={field.state.value}
                  onChange={(_, value) => field.handleChange(value)}
                  freeSolo
                  multiple
                  options={[]}
                />
                <FormHelperText>
                  Use an '*' to whitelist all values
                </FormHelperText>
              </FormField>
            )}
          />
          <form.Field
            name="attributes"
            validators={required}
            children={(field) => (
              <FormField label="Attributes" errors={field.state.meta.errors}>
                <Autocomplete
                  value={field.state.value}
                  onChange={(_, value) => field.handleChange(value)}
                  freeSolo
                  multiple
                  options={[]}
                />
                <FormHelperText>
                  Use an '*' to whitelist all values
                </FormHelperText>
              </FormField>
            )}
          />
          <Box>
            <Button size="md" type="submit">
              Next step
            </Button>
          </Box>
        </Stack>
      </form>
    </Stack>
  );
}
