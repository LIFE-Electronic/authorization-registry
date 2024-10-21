import { FormLabel, Input, Stack, Typography, Box } from "@mui/joy";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/__auth/new_policy_set")({
  component: Component,
});

function Component() {
  return (
    <div>
      <Typography level="h2">New policy set</Typography>
      <Stack paddingTop={2} spacing={1}>
        <Box>
          <FormLabel>Access subject</FormLabel>
          <Input />
        </Box>
        <Box>
          <FormLabel>Policy issuer</FormLabel>
          <Input />
        </Box>
      </Stack>
    </div>
  );
}
