import { Stepper, Step, StepIndicator, Typography } from "@mui/joy";

export function AddPolicySetStepper({ activeStep }: { activeStep: 1 | 2 | 3 }) {
  return (
    <Stepper sx={{ width: "100%" }}>
      <Step
        indicator={
          <StepIndicator
            variant={activeStep === 1 ? "solid" : "outlined"}
            color="neutral"
          >
            1
          </StepIndicator>
        }
      >
        <Typography
          textColor={activeStep === 1 ? "neutral.700" : "neutral.500"}
        >
          Define policy set
        </Typography>
      </Step>
      {/* <Step
        indicator={
          <StepIndicator variant={activeStep === 2 ? "solid" : "outlined"}>
            2
          </StepIndicator>
        }
      >
        <Typography
          textColor={activeStep === 2 ? "neutral.700" : "neutral.500"}
        >
          Define exception rules
        </Typography>
      </Step> */}
      <Step
        indicator={
          <StepIndicator variant={activeStep === 3 ? "solid" : "outlined"}>
            3
          </StepIndicator>
        }
      >
        <Typography
          textColor={activeStep === 3 ? "neutral.700" : "neutral.500"}
        >
          Review and Submit
        </Typography>
      </Step>
    </Stepper>
  );
}
