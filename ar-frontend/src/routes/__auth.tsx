import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { z } from "zod";
import { isAuthenticated, useAuth } from "../auth";
import { initLogin } from "../network/idp";

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
        replace: true,
        search: {
          ...search,
          token: undefined,
        },
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

  return <Outlet />;
}