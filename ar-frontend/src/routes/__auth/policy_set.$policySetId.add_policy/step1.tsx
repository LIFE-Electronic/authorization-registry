import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useForm } from "@tanstack/react-form";
import {
  Typography,
  Stack,
  Button,
  FormLabel,
  Select,
  Option,
  Input,
  FormControl,
  Autocomplete,
} from "@mui/joy";
import { AddPolicyStepper } from "../../../components/add-policy-stepper";

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step1",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const params = Route.useParams();

  const form = useForm<{
    actions: string[];
    resource_type: string;
    identifiers: string[];
    attributes: string[];
    service_providers: string[];
  }>({
    defaultValues: {
      actions: [],
      resource_type: "",
      identifiers: [],
      attributes: [],
      service_providers: [],
    },
    onSubmit: ({ value }) => {
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
      <Typography level="h4">Add policy</Typography>

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
            children={(field) => (
              <FormControl>
                <FormLabel>Actions</FormLabel>
                <Select
                  value={field.state.value}
                  onChange={(_, newValue) => field.handleChange(newValue)}
                  multiple
                >
                  <Option value="read">Read</Option>
                  <Option value="edit">Edit</Option>
                  <Option value="delete">Delete</Option>
                </Select>
              </FormControl>
            )}
          />
          <form.Field
            name="resource_type"
            children={(field) => (
              <FormControl>
                <FormLabel>Resource type</FormLabel>
                <Input
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
              </FormControl>
            )}
          />
          <form.Field
            name="identifiers"
            children={(field) => (
              <FormControl>
                <FormLabel>Identifiers</FormLabel>
                <Autocomplete
                  value={field.state.value}
                  onChange={(_, value) => field.handleChange(value)}
                  freeSolo
                  multiple
                  options={[]}
                />
              </FormControl>
            )}
          />
          <form.Field
            name="attributes"
            children={(field) => (
              <FormControl>
                <FormLabel>Attributes</FormLabel>
                <Autocomplete
                  value={field.state.value}
                  onChange={(_, value) => field.handleChange(value)}
                  freeSolo
                  multiple
                  options={[]}
                />
              </FormControl>
            )}
          />
          <Button type="submit">Submit</Button>
        </Stack>
      </form>
    </Stack>
  );
}
