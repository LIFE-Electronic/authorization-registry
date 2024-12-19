import { z } from "zod";
import { policySchema } from "./network/policy-set";

import policySetTemplatesJSON from "../policy-set-templates.json";

const policySetTemplateSchema = z.object({
  name: z.string(),
  policies: z.array(policySchema.omit({ id: true })),
  access_subject: z.string().optional(),
  policy_issuer: z.string().optional(),
});

export type PolicySetTemplate = z.infer<typeof policySetTemplateSchema>;

export let policySetTemplates: PolicySetTemplate[] = [];
try {
  policySetTemplates = z
    .array(policySetTemplateSchema)
    .parse(policySetTemplatesJSON);
} catch (e) {
  console.error(e);
}
