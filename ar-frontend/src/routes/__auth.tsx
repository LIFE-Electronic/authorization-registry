import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { z } from "zod";
import { isAuthenticated, useAuth } from "../auth";
import { initLogin } from "../network/idp";
import { Box } from "@mui/joy";
import { Logo } from "../components/logo";

const searchSchema = z
  .object({
    token: z.string().optional(),
  })
  .optional();

export const Route = createFileRoute("/__auth")({
  component: Component,
  validateSearch: searchSchema,
});

function Component() {
  const navigate = useNavigate();
  const search = Route.useSearch();
  const { token, setToken } = useAuth();

  useEffect(() => {
    if (search?.token && isAuthenticated(token)) {
      navigate({
        to: "/",
        replace: true,
        search: {},
      });
    }

    if (!isAuthenticated(token)) {
      if (search?.token) {
        setToken(search?.token);

        return;
      }

      initLogin();
    }
  }, [token, search?.token, search, setToken, navigate]);

  if (!isAuthenticated(token)) {
    return null;
  }

  return (
    <Box>
      <Box paddingY={2}>
        <Logo />
      </Box>
      <Outlet />
    </Box>
  );
}
