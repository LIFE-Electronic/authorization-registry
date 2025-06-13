import { required } from "@/form-field-validators";
import { Policy } from "@/network/policy-set";
import {
  Autocomplete,
  Box,
  Button,
  Card,
  FormHelperText,
  IconButton,
  Input,
  Option,
  Select,
  Stack,
  Typography,
} from "@mui/joy";
import { useForm } from "@tanstack/react-form";
import DeleteIcon from "@mui/icons-material/Delete";
import { FormField } from "./form-field";
import { ReactNode } from "@tanstack/react-router";

const defaultValues: Omit<Policy, "id"> = {
  actions: [],
  resource_type: "",
  identifiers: [],
  attributes: [],
  service_providers: [],
  rules: [],
};

export type PolicyFormFields = {
  actions: string[];
  resource_type: string;
  identifiers: string[];
  attributes: string[];
  service_providers: string[];
  rules: Policy["rules"];
};

export function PolicyForm({
  onSubmit,
  backButton,
  submitText,
  isSubmitPending,
  initialValues,
}: {
  onSubmit: (policy: PolicyFormFields) => void;
  backButton?: ReactNode;
  submitText: string;
  isSubmitPending?: boolean;
  initialValues?: PolicyFormFields;
}) {
  const form = useForm({
    defaultValues: initialValues || defaultValues,
    onSubmit: ({ value }) => {
      const policy = {
        ...value,
        rules:
          value.rules?.[0]?.effect === "Permit"
            ? value.rules
            : [{ effect: "Permit" as const }, ...value.rules],
      };
      onSubmit(policy);
    },
  });

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        e.stopPropagation();
      }}
    >
      <Stack spacing={2}>
        <Card>
          <Stack width="100%" spacing={2}>
            <Typography
              fontFamily="Inter Variable"
              fontSize="16px"
              color="primary"
              fontWeight={600}
            >
              Define policy
            </Typography>

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
                  <FormField
                    label="Resource type"
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
                  <FormField
                    label="Identifiers"
                    errors={field.state.meta.errors}
                  >
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
                  <FormField
                    label="Attributes"
                    errors={field.state.meta.errors}
                  >
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
            </Stack>
          </Stack>
        </Card>

        <form.Field name="rules" mode="array">
          {(field) => (
            <>
              {field.state.value.map((_rule, index) => (
                <Card key={index}>
                  <Stack spacing={2}>
                    <Box
                      display="flex"
                      justifyContent="space-between"
                      alignItems="center"
                    >
                      <Typography
                        fontFamily="Inter Variable"
                        fontSize="16px"
                        color="primary"
                        fontWeight={600}
                      >
                        Define exception rule ({index + 1})
                      </Typography>
                      <IconButton onClick={() => field.removeValue(index)}>
                        <DeleteIcon />
                      </IconButton>
                    </Box>

                    <Stack spacing={2}>
                      <form.Field
                        name={`rules[${index}].target.actions`}
                        children={(field) => (
                          <FormField
                            label="Actions"
                            errors={field.state.meta.errors}
                          >
                            <Select
                              value={field.state.value}
                              onChange={(_, newValue) =>
                                field.handleChange(newValue)
                              }
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
                        name={`rules[${index}].target.resource.type`}
                        children={(field) => (
                          <FormField
                            label="Resource type"
                            errors={field.state.meta.errors}
                          >
                            <Input
                              value={field.state.value}
                              onChange={(e) =>
                                field.handleChange(e.target.value)
                              }
                            />
                          </FormField>
                        )}
                      />

                      <form.Field
                        name={`rules[${index}].target.resource.identifiers`}
                        children={(field) => (
                          <FormField
                            label="Identifiers"
                            errors={field.state.meta.errors}
                          >
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
                        name={`rules[${index}].target.resource.attributes`}
                        children={(field) => (
                          <FormField
                            label="Attributes"
                            errors={field.state.meta.errors}
                          >
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
                    </Stack>
                  </Stack>
                </Card>
              ))}

              <Button
                variant="outlined"
                size="sm"
                onClick={() => {
                  field.pushValue({
                    effect: "Deny",
                    target: {
                      actions: [],
                      resource: {
                        type: "",
                        identifiers: [],
                        attributes: [],
                      },
                    },
                  });
                }}
              >
                Add another exception rule
              </Button>
            </>
          )}
        </form.Field>

        <Box paddingY={2} display="flex" gap={1} alignItems="center">
          <Button
            variant="solid"
            size="sm"
            type="submit"
            disabled={isSubmitPending}
            onClick={() => {
              form.handleSubmit();
            }}
          >
            {submitText}
          </Button>
          {backButton}
        </Box>
      </Stack>
    </form>
  );
}
