import { Box, CircularProgress } from "@mui/joy";
import { type ReactNode } from "react";

export function PageLoadingFallback({
  children,
  isLoading,
}: {
  children: ReactNode;
  isLoading: boolean;
}) {
  return isLoading ? (
    <Box
      width="100%"
      display="flex"
      justifyContent="center"
      alignItems="center"
      minHeight={200}
    >
      <CircularProgress color="neutral" />
    </Box>
  ) : (
    children
  );
}
