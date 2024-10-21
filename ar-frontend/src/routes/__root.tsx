import { Outlet, createRootRoute } from "@tanstack/react-router";
import { Box } from "@mui/joy";
import { z } from "zod";

const searchSchema = z
  .object({
    token: z.string().optional(),
  })
  .optional();

export const Route = createRootRoute({
  validateSearch: searchSchema,
  component: () => {
    return (
      <Box
        sx={{
          backgroundImage:
            "linear-gradient(to right bottom, #efe3eb, #fefefe, #fefefe, #efe3eb)",
          minHeight: "100vh",
          display: "flex",
          justifyContent: "center",
        }}
      >
        <Box sx={{ maxWidth: 900, width: 900, paddingX: 1 }}>
          <Outlet />
        </Box>
      </Box>
    );
  },
});
