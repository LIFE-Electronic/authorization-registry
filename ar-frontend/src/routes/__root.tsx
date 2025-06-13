import { Outlet, createRootRoute } from "@tanstack/react-router";
import { Box } from "@mui/joy";
import { z } from "zod";
import backgroundImage from "../assets/background.png";

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
          backgroundColor: "#f4f5f6",
          minHeight: "100vh",
          backgroundImage: `url(${backgroundImage})`,
          backgroundPosition: "top 200px left 0px",
          backgroundRepeat: "repeat-x",
        }}
      >
        <Outlet />
      </Box>
    );
  },
});
