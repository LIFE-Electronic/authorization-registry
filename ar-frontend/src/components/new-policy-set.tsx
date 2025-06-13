import {
  Alert,
  Box,
  Button,
  FormHelperText,
  FormLabel,
  IconButton,
  Input,
  Modal,
  ModalDialog,
  Option,
  Select,
  Stack,
  Step,
  stepClasses,
  StepIndicator,
  stepIndicatorClasses,
  Stepper,
  StepProps,
  Typography,
} from "@mui/joy";
import { LeftArrowIcon } from "@/icons/left-arrow-icons";
import { ReactNode } from "@tanstack/react-router";
import { Check } from "@/icons/check";
import { useState } from "react";
import { useCreatePolicySetContext } from "./create-policy-set-context";
import KeyboardArrowDown from "@mui/icons-material/KeyboardArrowDown";
import AddBoxIcon from "@mui/icons-material/AddBox";
import DeleteIcon from "@mui/icons-material/Delete";
import { usePolicySetTemplates } from "@/network/policy-set-templates";
import { PageLoadingFallback } from "./page-loading-fallback";
import { useForm } from "@tanstack/react-form";
import { required } from "@/form-field-validators";
import { FormField } from "./form-field";
import { PolicyCard } from "./policy-card";
import { PolicyForm } from "./policy-form";
import { ModalHeader } from "./modal-header";
import { AttentionIcon } from "@/icons/attention-icon";

export function StyledStep({ ...props }: {} & StepProps) {
  return (
    <Step
      sx={(theme) => ({
        typography: {
          // I apologize for the double ternary ;)
          color: props.active
            ? theme.vars.palette.primary[500]
            : props.completed
              ? theme.vars.palette.neutral[700]
              : theme.vars.palette.neutral[300],
          fontFamily: "Inter Variable",
          fontWeight: 600,
          letterSpacing: "-0.43px",
          textTransform: "none",
          fontSize: "14px",
        },
      })}
      indicator={
        <StepIndicator
          sx={(theme) => ({
            height: "20px",
            width: "20px",
            borderColor:
              props.active || props.completed
                ? theme.vars.palette.primary[500]
                : theme.vars.palette.neutral[300],
            borderWidth: 1,
            borderStyle: "solid",
            backgroundColor: props.completed
              ? theme.vars.palette.primary[300]
              : "white",
          })}
        >
          {props.completed && <Check />}
        </StepIndicator>
      }
      active
      {...props}
    />
  );
}

function Stage({
  step,
}: {
  step: (typeof NEW_POLICY_SET_STEPS)[keyof typeof NEW_POLICY_SET_STEPS];
}) {
  const currentStepIndex = NEW_POLICY_SET_STEPS.findIndex((l) => l === step);

  return (
    <Stepper
      sx={{
        "--Stepper-verticalGap": "32px",
        "--Step-connectorInset": "0px",
        [`& .${stepClasses.completed}`]: {
          [`& .${stepIndicatorClasses.root}`]: {
            borderColor: "primary.500",
            color: "primary.500",
          },
          "&::after": {
            bgcolor: "primary.300",
          },
        },
      }}
      orientation="vertical"
    >
      <StyledStep
        completed={currentStepIndex > 0}
        active={currentStepIndex === 0}
      >
        Prefill from template
      </StyledStep>
      <StyledStep
        completed={currentStepIndex > 1}
        active={currentStepIndex === 1}
      >
        Define policy step
      </StyledStep>
      <StyledStep
        completed={currentStepIndex > 2}
        active={currentStepIndex === 2}
      >
        Add policies
      </StyledStep>
      <StyledStep active={currentStepIndex === 3}>Review and submit</StyledStep>
    </Stepper>
  );
}

const NEW_POLICY_SET_STEPS = [
  "Prefill from template",
  "Define policy set",
  "Add policies",
  "Review and submit",
] as const;

export function NewPolicySetModalWrapper({
  step,
  children,
  onNext,
  onBack,
  nextDisabled,
  nextText,
}: {
  step: (typeof NEW_POLICY_SET_STEPS)[keyof typeof NEW_POLICY_SET_STEPS];
  children: ReactNode;
  onNext: () => void;
  onBack: () => void;
  nextDisabled?: boolean;
  nextText?: string;
}) {
  return (
    <Modal open={true}>
      <ModalDialog sx={{ padding: 0 }}>
        <Box>
          <ModalHeader caption="new" title="Add a new policy set" />

          <Box display="flex" sx={{ minHeight: "200px" }}>
            <Box
              display="flex"
              justifyContent="center"
              padding={4}
              boxSizing="border-box"
              width="246px"
            >
              <Stage step={step} />
            </Box>
            <Box
              sx={{
                backgroundColor: "#F4F5F6",
                display: "flex",
                flexGrow: 1,
                padding: 4,
                width: "1114px",
                height: "300px", // default
                "@media (min-height: 700px)": { height: "329px" },
                "@media (min-height: 800px)": { height: "429px" },
                "@media (min-height: 900px)": { height: "529px" },
                "@media (min-height: 1000px)": { height: "629px" },
                overflow: "auto",
              }}
            >
              {children}
            </Box>
          </Box>

          <Box
            sx={(theme) => ({
              borderTopStyle: "solid",
              borderColor: theme.vars.palette.neutral[100],
              borderWidth: "1px",
            })}
            height="96px"
            display="flex"
            alignItems="center"
          >
            <Box
              paddingX={4}
              display="flex"
              alignItems="center"
              justifyContent="space-between"
              width="100%"
            >
              <Button
                startDecorator={<LeftArrowIcon />}
                size="lg"
                sx={(theme) => ({
                  typography: {
                    textTransform: "none",
                    fontWeight: 300,
                    color: theme.vars.palette.neutral[900],
                    letterSpacing: "-0.43px",
                    fontSize: "16px",
                    paddingTop: "10px",
                    paddingBottom: "10px",
                    paddingLeft: "24px",
                    paddingRight: "24px",
                  },
                })}
                color="neutral"
                variant="plain"
                onClick={onBack}
              >
                Back
              </Button>
              <Button
                sx={{
                  height: "37px",
                }}
                disabled={nextDisabled}
                onClick={onNext}
              >
                {nextText || "Next"}
              </Button>
            </Box>
          </Box>
        </Box>
      </ModalDialog>
    </Modal>
  );
}

export function PrefillTemplateStep({
  onNextNavigation,
  onBack,
}: {
  onNextNavigation: () => void;
  onBack: () => void;
}) {
  const [template, setTemplate] = useState<number>();
  const { changeValue } = useCreatePolicySetContext();
  const { data: policySetTemplates, isLoading } = usePolicySetTemplates();

  function onNext() {
    if (template === -1) {
      changeValue({
        access_subject: "",
        policy_issuer: "",
        policies: [],
      });
    } else {
      if (template !== undefined) {
        const policySetTemplate = policySetTemplates?.[template];

        if (!policySetTemplate) {
          return;
        }

        changeValue({
          access_subject: policySetTemplate.access_subject || "",
          policy_issuer: policySetTemplate.policy_issuer || "",
          policies: policySetTemplate.policies,
        });
      }
    }

    onNextNavigation();
  }

  let description: string | undefined = undefined;

  if (
    template &&
    template !== -1 &&
    policySetTemplates?.[template]?.description
  ) {
    description = policySetTemplates?.[template]?.description;
  }

  return (
    <div>
      <NewPolicySetModalWrapper
        step="Prefill from template"
        onNext={onNext}
        onBack={onBack}
        nextDisabled={template === undefined}
      >
        <PageLoadingFallback isLoading={isLoading}>
          <Stack width="100%" spacing={2}>
            <Box>
              <FormLabel sx={{ fontSize: "16px" }}>
                Policy set template
              </FormLabel>
              <FormHelperText sx={{ fontSize: "16px" }}>
                Choose one of the policy set templates to pre-fill the wizard or
                start from scratch
              </FormHelperText>
            </Box>

            <Select
              indicator={<KeyboardArrowDown />}
              /* @ts-expect-error joy-ui is not smart enough to infer the type from dynamic options */
              onChange={(_, newValue) => setTemplate(newValue)}
            >
              {policySetTemplates?.map((ps, idx) => (
                <Option key={idx} value={idx} label={ps.name}>
                  {ps.name}
                </Option>
              ))}
              <Option value={-1}>Start from scratch</Option>
            </Select>
            {description && (
              <Box sx={{ maxWidth: 680 }}>
                <Stack paddingLeft={2} spacing={2} direction="row">
                  <Box>
                    <AttentionIcon />
                  </Box>
                  <Typography>{description}</Typography>
                </Stack>
              </Box>
            )}
          </Stack>
        </PageLoadingFallback>
      </NewPolicySetModalWrapper>
    </div>
  );
}

export function DefinePolicySetStep({
  onNextNavigation,
  onBack,
}: {
  onNextNavigation: () => void;
  onBack: () => void;
}) {
  const { value, changeValue } = useCreatePolicySetContext();

  const form = useForm({
    defaultValues: {
      access_subject: value.access_subject,
      policy_issuer: value.policy_issuer,
    },
    onSubmit: ({ value }) => {
      changeValue((oldValue) => ({ ...oldValue, ...value }));

      onNextNavigation();
    },
  });

  function onNext() {
    form.handleSubmit();
  }

  return (
    <div>
      <NewPolicySetModalWrapper
        step="Define policy set"
        onNext={onNext}
        onBack={onBack}
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
            <Box>
              <FormLabel sx={{ fontSize: "16px" }}>Define policy set</FormLabel>
              <FormHelperText sx={{ fontSize: "16px" }}>
                Choose the policy issuer and the access subject
              </FormHelperText>
            </Box>
            <form.Field
              name="policy_issuer"
              validators={required}
              children={(field) => (
                <FormField
                  label="Policy issuer"
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
          </Stack>
        </form>
      </NewPolicySetModalWrapper>
    </div>
  );
}

export function AddPoliciesStep({
  onBack,
  onNextNavigation,
}: {
  onBack: () => void;
  onNextNavigation: () => void;
}) {
  const [addPolicyFormOpen, setAddPolicyFormOpen] = useState(false);
  const { value: policySet, changeValue: changePolicySetValue } =
    useCreatePolicySetContext();

  return (
    <div>
      <NewPolicySetModalWrapper
        step="Add policies"
        onNext={onNextNavigation}
        onBack={onBack}
      >
        <Stack width="100%" spacing={2}>
          <Box>
            <FormLabel sx={{ fontSize: "16px" }}>Add policies</FormLabel>
            <FormHelperText sx={{ fontSize: "16px" }}>
              Add one or more policies to the policy set
            </FormHelperText>
          </Box>
          {addPolicyFormOpen ? (
            <Box width="100%">
              <PolicyForm
                onSubmit={(policy) => {
                  changePolicySetValue((old) => ({
                    ...old,
                    policies: [...old.policies, policy],
                  }));

                  setAddPolicyFormOpen(false);
                }}
                submitText="Add policy"
                backButton={
                  <Button
                    variant="plain"
                    size="sm"
                    color="neutral"
                    onClick={() => setAddPolicyFormOpen(false)}
                  >
                    Back to add policies step
                  </Button>
                }
              />
            </Box>
          ) : (
            <Stack spacing={2} width="100%">
              <Box>
                <Button
                  onClick={() => setAddPolicyFormOpen(true)}
                  startDecorator={<AddBoxIcon />}
                  variant="outlined"
                  size="sm"
                >
                  Add policy
                </Button>
              </Box>
              <Box flexWrap="wrap" display="flex" gap={1} paddingBottom={2}>
                {policySet.policies.map((policy, idx) => (
                  <Box
                    key={idx}
                    display="flex"
                    width={{
                      xs: "100%",
                      sm: "47%",
                      md: "32%",
                    }}
                    maxWidth={{
                      xs: "100%",
                      sm: "47%",
                      md: "32%",
                    }}
                  >
                    <PolicyCard
                      detailed
                      policy={policy}
                      actions={
                        <Box>
                          <IconButton
                            color="danger"
                            onClick={() => {
                              changePolicySetValue((old) => ({
                                ...old,
                                policies: old.policies.filter(
                                  (_, currentIdx) => currentIdx !== idx,
                                ),
                              }));
                            }}
                          >
                            <DeleteIcon />
                          </IconButton>
                        </Box>
                      }
                    />
                  </Box>
                ))}
              </Box>
            </Stack>
          )}
        </Stack>
      </NewPolicySetModalWrapper>
    </div>
  );
}

export function ReviewAndSubmitStep({
  onNext,
  nextPending,
  onBack,
  error,
}: {
  onNext: () => void;
  nextPending?: boolean;
  onBack: () => void;
  error?: string;
}) {
  const { value } = useCreatePolicySetContext();

  return (
    <NewPolicySetModalWrapper
      step="Review and submit"
      onBack={onBack}
      onNext={onNext}
      nextText="Save policy set"
      nextDisabled={nextPending}
    >
      <Stack width="100%" spacing={2}>
        <Box>
          <FormLabel sx={{ fontSize: "16px" }}>Review and submit</FormLabel>
          <FormHelperText sx={{ fontSize: "16px" }}>
            View the overview, create the policy set or go back and make some
            modifications
          </FormHelperText>
        </Box>
        {error && (
          <Box>
            <Alert color="danger">
              <Box>{error}</Box>
            </Alert>
          </Box>
        )}
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
          <Stack direction="row" spacing={1} paddingBottom={2}>
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
          </Stack>
        </Box>
      </Stack>
    </NewPolicySetModalWrapper>
  );
}
