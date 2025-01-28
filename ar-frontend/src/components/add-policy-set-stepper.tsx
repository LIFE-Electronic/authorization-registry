import { Stepper, Step, StepIndicator, Typography } from "@mui/joy";

const steps = [
  "Prefill from template",
  "Define policy set",
  "Add policies",
  "Review and submit",
] as const;

export function AddPolicySetStepper({
  activeStep,
}: {
  activeStep: (typeof steps)[keyof typeof steps];
}) {
  return (
    <Stepper sx={{ width: "100%" }}>
      {steps
        .map((s, idx) => (
          <Step
            key={idx}
            indicator={
              <StepIndicator
                variant={activeStep === s ? "solid" : "outlined"}
                color="neutral"
              >
                {idx + 1}
              </StepIndicator>
            }
          >
            <Typography
              textColor={activeStep === s ? "neutral.700" : "neutral.500"}
            >
              {s}
            </Typography>
          </Step>
        ))}
    </Stepper>
  );
}
