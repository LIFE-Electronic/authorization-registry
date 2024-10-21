import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { z } from "zod";
import { AddPolicyStepper } from "../../../components/add-policy-stepper";
import {
  Autocomplete,
  Button,
  Divider,
  FormControl,
  FormLabel,
  Input,
  Stack,
  Select,
  Option,
} from "@mui/joy";
import { useForm } from "@tanstack/react-form";

const searchSchema = z.object({
  actions: z.array(z.string()),
  resource_type: z.string(),
  identifiers: z.array(z.string()),
  attributes: z.array(z.string()),
  service_providers: z.array(z.string()),
  rules: z.array(
    z.object({
      resource_type: z.string(),
      identifiers: z.array(z.string()),
      attributes: z.array(z.string()),
      actions: z.array(z.string()),
    }),
  ),
});

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step2",
)({
  component: Component,
  validateSearch: searchSchema,
});

function Component() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const params = Route.useParams();

  const form = useForm<{
    resource_type: string;
    identifiers: string[];
    attributes: string[];
    actions: string[];
  }>({
    defaultValues: {
      resource_type: "",
      identifiers: [],
      attributes: [],
      actions: [],
    },
    onSubmit: ({ value }) => {
      navigate({
        to: "/policy_set/$policySetId/add_policy/step2",
        params,
        search: {
          ...search,
          rules: [...search.rules, value],
        },
      });
    },
  });

  return (
    <Stack spacing={3}>
      <AddPolicyStepper activeStep={2} />
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
          <Button type="submit" variant="outlined">
            Add exception
          </Button>
        </Stack>
      </form>

      <Divider />

      <Stack direction="row">
        <Button
          onClick={() =>
            navigate({
              to: "/policy_set/$policySetId/add_policy/step3",
              params,
              search,
            })
          }
        >
          Review and submit
        </Button>
      </Stack>
    </Stack>
  );
}
