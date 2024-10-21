import { Alert, Box, Button, Stack, Typography } from "@mui/joy";
import { ErrorResponse } from "../network/fetch";
import { useQueryClient } from "@tanstack/react-query";

export function CatchBoundary({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  const queryClient = useQueryClient();
  const message =
    error instanceof ErrorResponse
      ? error.message
      : "Something unexpected went wrong. Try again at a later point.";

  return (
    <Alert color="danger">
      <Stack spacing={4}>
        <Typography>{message}</Typography>
        <Box>
          <Button
            variant="soft"
            color="neutral"
            onClick={() => {
              queryClient.invalidateQueries();
              reset();
            }}
          >
            Reload
          </Button>
        </Box>
      </Stack>
    </Alert>
  );
}
