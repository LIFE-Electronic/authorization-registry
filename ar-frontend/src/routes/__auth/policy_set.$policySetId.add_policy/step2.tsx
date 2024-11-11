import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Stack } from "@mui/joy";

import { useAddPolicyContext } from "../../../components/add-policy-context";
import { Step2, Step2FormFields } from "@/components/add-policy";

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy/step2",
)({
  component: Component,
});

function Component() {
  const search = Route.useSearch();
  const navigate = useNavigate();
  const params = Route.useParams();
  const { changeValue } = useAddPolicyContext();

  function onSubmit({ value }: { value: Step2FormFields }) {
    changeValue((oldValue) => ({
      ...oldValue,
      rules: [
        ...oldValue.rules,
        {
          effect: "Deny",
          target: {
            actions: value.actions,
            resource: {
              type: value.resource_type,
              identifiers: value.identifiers,
              attributes: value.attributes,
            },
          },
        },
      ],
    }));
  }

  function onBack() {
    navigate({
      to: "/policy_set/$policySetId/add_policy/step1",
      params,
    });
  }

  function onNext() {
    navigate({
      to: "/policy_set/$policySetId/add_policy/step3",
      params,
      search,
    });
  }

  return (
    <Stack spacing={3}>
      <Step2 onSubmit={onSubmit} onBack={onBack} onNext={onNext} />
    </Stack>
  );
}
