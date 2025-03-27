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
  Divider,
  Typography,
  Card,
  IconButton,
  Alert,
} from "@mui/joy";
import DeleteIcon from "@mui/icons-material/Delete";
import { AddEditPolicyStepper } from "@/components/add-edit-policy-stepper";
import { required } from "@/form-field-validators";
import { FormField } from "@/components/form-field";
import { useAddPolicyContext } from "@/components/add-edit-policy-context";
import { PolicyCard } from "./policy-card";
import { Policy } from "@/network/policy-set";
import { Fragment } from "react/jsx-runtime";
import { ErrorResponse } from "@/network/fetch";
import { arrayValueToDisplay } from "@/array-to-display";

export type Step1FormFields = {
  actions: string[];
  resource_type: string;
  identifiers: string[];
  attributes: string[];
  service_providers: string[];
};

export function Step1({ onSubmit }: { onSubmit: () => void }) {
  const { value, changeValue } = useAddPolicyContext();

  const form = useForm<Step1FormFields>({
    defaultValues: value,
    onSubmit: ({ value }) => {
      changeValue((oldValue) => ({ ...oldValue, ...value }));
      onSubmit();
    },
  });

  return (
    <Stack spacing={3}>
      <AddEditPolicyStepper activeStep={1} />

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
                  <Option value="Read">Read</Option>
                  <Option value="Edit">Edit</Option>
                  <Option value="Delete">Delete</Option>
                  <Option value="Create">Create</Option>
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
            children={(field) => (
              <FormField
                errors={field.state.meta.errors}
                label="Service providers"
              >
                <Autocomplete
                  clearOnBlur
                  value={field.state.value}
                  onChange={(_, value) => {
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

export type Step2FormFields = {
  resource_type: string;
  identifiers: string[];
  attributes: string[];
  actions: string[];
};

export function Step2({
  onNext,
  onBack,
}: {
  onNext: () => void;
  onBack: () => void;
}) {
  const { changeValue, value } = useAddPolicyContext();

  const form = useForm<Step2FormFields>({
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
      <AddEditPolicyStepper activeStep={2} />
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
          {value.rules.map((r, idx) =>
            r.effect === "Deny" ? (
              <Card key={idx}>
                <Box
                  display="flex"
                  justifyContent="space-between"
                  sx={{ whiteSpace: "pre" }}
                >
                  <Stack>
                    <Typography level="body-sm">
                      Actions: {arrayValueToDisplay(r.target.actions)}
                    </Typography>
                    <Typography level="body-sm">
                      Resource type: {r.target.resource.type}
                    </Typography>
                    <Typography level="body-sm">
                      Identifiers:{" "}
                      {arrayValueToDisplay(r.target.resource.identifiers)}
                    </Typography>
                    <Typography level="body-sm">
                      Attributes:{" "}
                      {arrayValueToDisplay(r.target.resource.attributes)}
                    </Typography>
                  </Stack>
                  <Box>
                    <IconButton
                      onClick={() =>
                        changeValue((oldValue) => ({
                          ...oldValue,
                          rules: oldValue.rules.filter(
                            (_, idx2) => idx2 !== idx,
                          ),
                        }))
                      }
                    >
                      <DeleteIcon />
                    </IconButton>
                  </Box>
                </Box>
              </Card>
            ) : (
              <Fragment key={idx}></Fragment>
            ),
          )}
        </Stack>
      )}

      <Stack direction="row" spacing={1}>
        <Button variant="outlined" onClick={onBack}>
          Previous step
        </Button>
        <Button onClick={onNext}>Review and submit</Button>
      </Stack>
    </Stack>
  );
}

export function Step3({
  onSubmit,
  isSubmitting,
  onBack,
  error,
}: {
  onSubmit: ({ policy }: { policy: Omit<Policy, "id"> }) => void;
  isSubmitting?: boolean;
  onBack: () => void;
  error?: ErrorResponse | null;
}) {
  const { value } = useAddPolicyContext();
  const policy = {
    ...value,
    rules:
      value.rules?.[0]?.effect === "Permit"
        ? value.rules
        : [{ effect: "Permit" as const }, ...value.rules],
  };

  return (
    <Stack spacing={3}>
      <AddEditPolicyStepper activeStep={3} />
      {error && (
        <Box paddingY={2}>
          <Alert color="danger">
            <Box>{error.message}</Box>
          </Alert>
        </Box>
      )}
      <PolicyCard policy={policy} />

      <Stack direction="row" spacing={1}>
        <Button variant="outlined" onClick={onBack}>
          Previous step
        </Button>
        <Button disabled={isSubmitting} onClick={() => onSubmit({ policy })}>
          Submit policy
        </Button>
      </Stack>
    </Stack>
  );
}
