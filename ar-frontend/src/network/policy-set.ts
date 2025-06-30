import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { baseAPIUrl, ErrorResponse, useAuthenticatedFetch } from "./fetch";
import { z } from "zod";

export type CreatePolicySet = {
  access_subject: string;
  policy_issuer: string;
  policies: Omit<Policy, "id">[];
};

export const policySchema = z.object({
  id: z.string(),
  actions: z.array(z.string()),
  identifiers: z.array(z.string()),
  resource_type: z.string(),
  attributes: z.array(z.string()),
  service_providers: z.array(z.string()),
  rules: z.array(
    z.union([
      z.object({
        effect: z.literal("Permit"),
      }),
      z.object({
        effect: z.literal("Deny"),
        target: z.object({
          actions: z.array(z.string()),
          resource: z.object({
            type: z.string(),
            attributes: z.array(z.string()),
            identifiers: z.array(z.string()),
          }),
        }),
      }),
    ]),
  ),
});

const policySetWithPoliciesSchema = z.object({
  policy_set_id: z.string(),
  policies: z.array(policySchema),
  access_subject: z.string(),
  policy_issuer: z.string(),
});

const policySetsResponse = z.object({
  data: z.array(policySetWithPoliciesSchema),
  pagination: z.object({
    total_count: z.number(),
  }),
});

export type Policy = z.infer<typeof policySchema>;
export type PolicySetWithPolicies = z.infer<typeof policySetWithPoliciesSchema>;

export function useAdminPolicySet({ policySetId }: { policySetId: string }) {
  const authenticatedFetch = useAuthenticatedFetch();

  return useQuery({
    throwOnError: true,
    queryKey: ["admin", "policy-sets", policySetId],
    queryFn: async function () {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set/${policySetId}`,
      );
      const json = await response.json();

      try {
        return policySetWithPoliciesSchema.parse(json);
      } catch (e) {
        console.error(e);
      }
    },
  });
}

export function usePolicySet({ policySetId }: { policySetId: string }) {
  const authenticatedFetch = useAuthenticatedFetch();

  return useQuery({
    throwOnError: true,
    queryKey: ["member", "policy-sets", policySetId],
    queryFn: async function () {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/policy-set/${policySetId}`,
      );
      const json = await response.json();

      try {
        return policySetWithPoliciesSchema.parse(json);
      } catch (e) {
        console.error(e);
      }
    },
  });
}

export function useAdminPolicySets({
  q,
  skip,
  limit,
}: {
  q?: string;
  skip?: number;
  limit?: number;
}) {
  const search = new URLSearchParams();
  const authenticatedFetch = useAuthenticatedFetch();

  if (q) {
    search.append("q", q);
  }

  if (skip) {
    search.append("skip", skip.toString());
  }

  if (limit) {
    search.append("limit", limit.toString());
  }

  return useQuery({
    throwOnError: true,
    queryKey: ["admin", "policy-sets", search.toString()],
    queryFn: async function () {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set?${search}`,
      );
      const json = await response.json();

      try {
        return policySetsResponse.parse(json);
      } catch (e) {
        console.error(e);
      }
    },
  });
}

export function usePolicySets({
  q,
  skip,
  limit,
}: {
  q?: string;
  skip?: number;
  limit?: number;
}) {
  const search = new URLSearchParams();
  const authenticatedFetch = useAuthenticatedFetch();

  if (q) {
    search.append("q", q);
  }

  if (skip) {
    search.append("skip", skip.toString());
  }

  if (limit) {
    search.append("limit", limit.toString());
  }

  return useQuery({
    throwOnError: true,
    queryKey: ["member", "policy-sets", search.toString()],
    queryFn: async function () {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/policy-set?${search}`,
      );
      const json = await response.json();

      try {
        return policySetsResponse.parse(json);
      } catch (e) {
        console.error(e);
      }
    },
  });
}

export function useAddAdminPolicyToPolicySet({
  policySetId,
}: {
  policySetId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, { policy: Omit<Policy, "id"> }>({
    mutationFn: async ({ policy }: { policy: Omit<Policy, "id"> }) => {
      await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set/${policySetId}/policy`,
        {
          method: "POST",
          body: JSON.stringify({
            target: {
              actions: policy.actions,
              environment: {
                serviceProviders: policy.service_providers,
              },
              resource: {
                type: policy.resource_type,
                identifiers: policy.identifiers,
                attributes: policy.attributes,
              },
            },
            rules: policy.rules,
          }),
          headers: {
            "Content-Type": "application/json",
          },
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets", policySetId],
      });
    },
  });
}

export function useAddPolicyToPolicySet({
  policySetId,
}: {
  policySetId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, { policy: Omit<Policy, "id"> }>({
    mutationFn: async ({ policy }: { policy: Omit<Policy, "id"> }) => {
      await authenticatedFetch(
        `${baseAPIUrl}/policy-set/${policySetId}/policy`,
        {
          method: "POST",
          body: JSON.stringify({
            target: {
              actions: policy.actions,
              environment: {
                serviceProviders: policy.service_providers,
              },
              resource: {
                type: policy.resource_type,
                identifiers: policy.identifiers,
                attributes: policy.attributes,
              },
            },
            rules: policy.rules,
          }),
          headers: {
            "Content-Type": "application/json",
          },
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets", policySetId],
      });
    },
  });
}

export function useReplaceAdminPolicyToPolicySet({
  policySetId,
  policyId,
}: {
  policySetId: string;
  policyId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, { policy: Omit<Policy, "id"> }>({
    mutationFn: async ({ policy }: { policy: Omit<Policy, "id"> }) => {
      await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set/${policySetId}/policy/${policyId}`,
        {
          method: "PUT",
          body: JSON.stringify({
            target: {
              actions: policy.actions,
              environment: {
                serviceProviders: policy.service_providers,
              },
              resource: {
                type: policy.resource_type,
                identifiers: policy.identifiers,
                attributes: policy.attributes,
              },
            },
            rules: policy.rules,
          }),
          headers: {
            "Content-Type": "application/json",
          },
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets", policySetId],
      });

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-set", policySetId, "policy", policyId],
      });
    },
  });
}

export function useReplacePolicyToPolicySet({
  policySetId,
  policyId,
}: {
  policySetId: string;
  policyId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, { policy: Omit<Policy, "id"> }>({
    mutationFn: async ({ policy }: { policy: Omit<Policy, "id"> }) => {
      await authenticatedFetch(
        `${baseAPIUrl}/policy-set/${policySetId}/policy/${policyId}`,
        {
          method: "PUT",
          body: JSON.stringify({
            target: {
              actions: policy.actions,
              environment: {
                serviceProviders: policy.service_providers,
              },
              resource: {
                type: policy.resource_type,
                identifiers: policy.identifiers,
                attributes: policy.attributes,
              },
            },
            rules: policy.rules,
          }),
          headers: {
            "Content-Type": "application/json",
          },
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets", policySetId],
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-set", policySetId, "policy", policyId],
      });
    },
  });
}

export function useDeleteAdminPolicyFromPolicySet({
  policySetId,
}: {
  policySetId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, { policyId: string }>({
    mutationFn: async ({ policyId }: { policyId: string }) => {
      await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set/${policySetId}/policy/${policyId}`,
        {
          method: "DELETE",
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets", policySetId],
      });
    },
  });
}

export function useDeletePolicyFromPolicySet({
  policySetId,
}: {
  policySetId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, { policyId: string }>({
    mutationFn: async ({ policyId }: { policyId: string }) => {
      await authenticatedFetch(
        `${baseAPIUrl}/policy-set/${policySetId}/policy/${policyId}`,
        {
          method: "DELETE",
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets", policySetId],
      });
    },
  });
}

export function useAdminGetPolicy({
  policyId,
  policySetId,
}: {
  policyId: string;
  policySetId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();

  return useQuery({
    throwOnError: true,
    queryKey: ["admin", "policy-set", policySetId, "policy", policyId],
    queryFn: async () => {
      const response = await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set/${policySetId}/policy/${policyId}`,
      );
      const json = await response.json();
      return policySchema.parse(json);
    },
  });
}

export function useDeleteAdminPolicySet({
  policySetId,
}: {
  policySetId: string;
}) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, void>({
    mutationFn: async () => {
      await authenticatedFetch(
        `${baseAPIUrl}/admin/policy-set/${policySetId}`,
        {
          method: "DELETE",
        },
      );

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets", policySetId],
      });
    },
  });
}

export function useDeletePolicySet({ policySetId }: { policySetId: string }) {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, void>({
    mutationFn: async () => {
      await authenticatedFetch(`${baseAPIUrl}/policy-set/${policySetId}`, {
        method: "DELETE",
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets"],
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets", policySetId],
      });
    },
  });
}

export function useAdminCreatePolicySet() {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, CreatePolicySet>({
    mutationFn: async (policySet: CreatePolicySet) => {
      await authenticatedFetch(`${baseAPIUrl}/admin/policy-set`, {
        method: "POST",
        body: JSON.stringify({
          target: {
            accessSubject: policySet.access_subject,
          },
          licences: [],
          maxDelegationDepth: 1,
          policies: policySet.policies.map((p) => ({
            rules: p.rules,
            target: {
              actions: p.actions,
              resource: {
                type: p.resource_type,
                identifiers: p.identifiers,
                attributes: p.attributes,
              },
              environment: {
                serviceProviders: p.service_providers,
              },
            },
          })),
          policyIssuer: policySet.policy_issuer,
        }),
        headers: {
          "Content-Type": "application/json",
        },
      });

      queryClient.invalidateQueries({
        queryKey: ["admin", "policy-sets"],
      });
    },
  });
}

export function useCreatePolicySet() {
  const authenticatedFetch = useAuthenticatedFetch();
  const queryClient = useQueryClient();

  return useMutation<void, ErrorResponse, CreatePolicySet>({
    mutationFn: async (policySet: CreatePolicySet) => {
      await authenticatedFetch(`${baseAPIUrl}/policy-set`, {
        method: "POST",
        body: JSON.stringify({
          target: {
            accessSubject: policySet.access_subject,
          },
          licences: [],
          maxDelegationDepth: 1,
          policies: policySet.policies.map((p) => ({
            rules: p.rules,
            target: {
              actions: p.actions,
              resource: {
                type: p.resource_type,
                identifiers: p.identifiers,
                attributes: p.attributes,
              },
              environment: {
                serviceProviders: p.service_providers,
              },
            },
          })),
          policyIssuer: policySet.policy_issuer,
        }),
        headers: {
          "Content-Type": "application/json",
        },
      });

      queryClient.invalidateQueries({
        queryKey: ["member", "policy-sets"],
      });
    },
  });
}
