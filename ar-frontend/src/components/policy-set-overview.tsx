import { Box, Button, Typography } from "@mui/joy";

export function PolicySetOverviewHeader({
  onNewPolicySet,
}: {
  onNewPolicySet: () => void;
}) {
  return (
    <Box paddingY={4} display="flex" gap={4}>
      <Typography level="h2">Policy sets</Typography>
      <div>
        <Button
          sx={{
            borderRadius: "8px",
            height: "43px",
            boxShadow: "0px 0px 36px 0px #FF358340",
            backgroundColor: "#007EFF",
          }}
          onClick={onNewPolicySet}
        >
          New policy set
        </Button>
      </div>
    </Box>
  );
}
