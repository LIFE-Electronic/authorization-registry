import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { AddPolicyStepper } from "../../../components/add-policy-stepper";
import {
  Autocomplete,
  Button,
  Divider,
  Input,
  Stack,
  Select,
  Option,
  Box,
  FormHelperText,
  Card,
  Typography,
  IconButton,
} from "@mui/joy";
import { useForm } from "@tanstack/react-form";
import { FormField } from "../../../components/form-field";
import { required } from "../../../form-field-validators";
import { useAddPolicyContext } from "../policy_set.$policySetId.add_policy";
import DeleteIcon from '@mui/icons-material/Delete';

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step2",
)({
  component: Component,
});

function Component() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const params = Route.useParams();
  const { changeValue, value } = useAddPolicyContext();

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
      changeValue((oldValue) => ({
        ...oldValue,
        rules: [
          ...oldValue.rules,
          {
            effect: "Deny",
            target: {
              actions: value.actions,
              resource: {
                type: value.resource_type,
                identifiers: value.identifiers,
                attributes: value.attributes,
              },
            },
          },
        ],
      }));
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
          <Button type="submit" variant="outlined">
            Add exception
          </Button>
        </Stack>
      </form>

      <Divider />
        {value.rules.length > 0 && (
          <Stack spacing={1}>
            {value.rules.map((r, idx) => (
              r.effect === "Deny" ? (
                <Card key={idx}>
                  <Box display="flex" justifyContent="space-between">
                    <Stack>
                      <Typography level="body-sm">Actions: {r.target.actions}</Typography>
                      <Typography level="body-sm">Resource type: {r.target.resource.type}</Typography>
                      <Typography level="body-sm">Identifiers: {r.target.resource.identifiers}</Typography>
                      <Typography level="body-sm">Attributes: {r.target.resource.attributes}</Typography>
                    </Stack>
                    <Box>
                    <IconButton onClick={() => changeValue(oldValue => ({
                      ...oldValue,
                      rules: oldValue.rules.filter((_, idx2) => idx2 !== idx),
                    }))}>
                      <DeleteIcon />
                    </IconButton>
                    </Box>
                  </Box>
                </Card>
              ) : <></>
            ))}
          </Stack>
        )}

      <Stack direction="row" spacing={1}>
        <Button
          variant="outlined"
          onClick={() => {
            navigate({
              to: "/policy_set/$policySetId/add_policy/step1",
              params,
            });
          }}
        >
          Back
        </Button>
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
