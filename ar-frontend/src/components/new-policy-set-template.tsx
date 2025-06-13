import {
  Box,
  Button,
  Modal,
  ModalDialog,
  stepClasses,
  stepIndicatorClasses,
  Stepper,
} from "@mui/joy";
import { ReactNode } from "@tanstack/react-router";
import { ModalHeader } from "./modal-header";
import { StyledStep } from "./new-policy-set";
import { LeftArrowIcon } from "@/icons/left-arrow-icons";

function Stage({
  step,
}: {
  step: (typeof NEW_POLICY_SET_TEMPLATE_STEPS)[keyof typeof NEW_POLICY_SET_TEMPLATE_STEPS];
}) {
  const currentStepIndex = NEW_POLICY_SET_TEMPLATE_STEPS.findIndex(
    (l) => l === step,
  );

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
        Define policy step
      </StyledStep>
      <StyledStep
        completed={currentStepIndex > 1}
        active={currentStepIndex === 1}
      >
        Add policies
      </StyledStep>
      <StyledStep active={currentStepIndex === 2}>Review and submit</StyledStep>
    </Stepper>
  );
}

const NEW_POLICY_SET_TEMPLATE_STEPS = [
  "Define policy set template",
  "Add policies",
  "Review and submit",
] as const;

export function NewPolicySetTemplateModalWrapper({
  step,
  children,
  onNext,
  onBack,
  nextDisabled,
  nextText,
}: {
  step: (typeof NEW_POLICY_SET_TEMPLATE_STEPS)[keyof typeof NEW_POLICY_SET_TEMPLATE_STEPS];
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
          <ModalHeader caption="new" title="Add a new policy set template" />

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
