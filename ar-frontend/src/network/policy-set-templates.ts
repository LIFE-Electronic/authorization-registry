import { z } from "zod";
import { policySchema, type Policy } from "@/network/policy-set";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { baseAPIUrl, ErrorResponse, useAuthenticatedFetch } from "./fetch";

const policySetTemplateSchema = z.object({
  name: z.string(),
  policies: z.array(policySchema.omit({ id: true })),
  access_subject: z.string().optional().nullable(),
  policy_issuer: z.string().optional().nullable(),
  description: z.string().optional().nullable(),
  id: z.string(),
});

export function usePolicySetTemplates() {
  const authenticatedFetch = useAuthenticatedFetch();

  return useQuery({
    throwOnError: true,
    queryKey: ["policy-set-template"],
    queryFn: async function () {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/policy-set-template`,
      );
      const json = await response.json();

      try {
        return z.array(policySetTemplateSchema).parse(json);
      } catch (e) {
        console.error(e);
      }
    },
  });
}

export type CreatePolicySetTemplate = {
  name: string;
  description: string;
  policies: Omit<Policy, "id">[];
  access_subject?: string;
  policy_issuer?: string;
};

export function useAdminCreatePolicySetTemplate() {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async function (policySetTemplate: CreatePolicySetTemplate) {
      await authenticatedFetch(`${baseAPIUrl}/admin/policy-set-template`, {
        method: "POST",
        body: JSON.stringify(policySetTemplate),
        headers: {
          "Content-Type": "application/json",
        },
      });

      queryClient.invalidateQueries({ queryKey: ["policy-set-template"] });
    },
  });
}

export function useAdminDeletePolicySetTemplate() {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, string>({
    mutationFn: async function (id: string) {
      await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set-template/${id}`,
        {
          method: "DELETE",
        },
      );

      queryClient.invalidateQueries({ queryKey: ["policy-set-template"] });
      queryClient.invalidateQueries({ queryKey: ["policy-set-template", id] });
    },
  });
}

export function usePolicySetTemplate({ id }: { id: string }) {
  const authenticatedFetch = useAuthenticatedFetch();

  return useQuery({
    throwOnError: true,
    queryKey: ["policy-set-template", id],
    queryFn: async function () {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/policy-set-template/${id}`,
      );
      const json = await response.json();

      try {
        return policySetTemplateSchema.parse(json);
      } catch (e) {
        console.error(e);
      }
    },
  });
}
