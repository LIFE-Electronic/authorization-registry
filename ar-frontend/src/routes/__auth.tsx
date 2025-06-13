import { createFileRoute, Outlet, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";
import { z } from "zod";
import { getTokenContent, isAuthenticated, useAuth } from "../auth";
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
    if (isAuthenticated(token) && token) {
      const tokenContent = getTokenContent(token);
      navigate({
        to: tokenContent.realm_access_roles.includes("dexspace_admin")
          ? "/admin/policy_set"
          : "/member",
        replace: true,
        search: {
          ...search,
          // @ts-ignore
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

  if (!isAuthenticated(token) || !token) {
    return null;
  }

  return <Outlet />;
}
