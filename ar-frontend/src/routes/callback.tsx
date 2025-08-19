import { getTokenContent, useAuthStore } from "@/auth";
import { createFileRoute, redirect } from "@tanstack/react-router";
import { z } from "zod";

const validateSearch = z.object({
  token: z.string(),
  state: z.string().nullable().optional(),
});

export const Route = createFileRoute("/callback")({
  validateSearch: validateSearch,
  component: Component,
  beforeLoad: ({ search }) => {
    useAuthStore.getState().setToken(search.token);
    const tokenContent = getTokenContent(search.token);

    if (search.state) {
      throw redirect({
        to: search.state,
      });
    }

    throw redirect({
      to: tokenContent.realm_access_roles.includes("dexspace_admin")
        ? "/admin/policy_set"
        : "/member",
    });
  },
});

function Component() {
  return null;
}
