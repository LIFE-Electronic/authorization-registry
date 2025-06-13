import { ModalHeader } from "@/components/modal-header";
import { PolicyForm } from "@/components/policy-form";
import {
  Policy,
  usePolicySet,
  useReplacePolicyToPolicySet,
} from "@/network/policy-set";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import {
  Stack,
  Button,
  Box,
  Modal,
  ModalOverflow,
  ModalDialog,
  Alert,
} from "@mui/joy";
import { createFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createFileRoute(
  "/__auth/member/policy_set/$policySetId/edit_policy/$policyId",
)({
  component: Component,
});

function Component() {
  const params = Route.useParams();
  const navigate = useNavigate();

  const {
    mutateAsync: replacePolicy,
    isPending,
    error,
  } = useReplacePolicyToPolicySet({
    policyId: params.policyId,
    policySetId: params.policySetId,
  });

  function onSubmit(policy: Omit<Policy, "id">) {
    replacePolicy({ policy }).then(() => {
      navigate({
        to: "/member/policy_set/$policySetId",
        params: {
          policySetId: params.policySetId,
        },
      });
    });
  }

  const { data: policySet, isLoading } = usePolicySet({
    policySetId: params.policySetId,
  });

  const policy = policySet?.policies.find(({ id }) => id === params.policyId);

  if (!policy || isLoading) {
    return <div />;
  }

  return (
    <Modal open={true} onClose={() => navigate({ to: "/admin" })}>
      <ModalOverflow>
        <ModalDialog sx={{ padding: 0 }} size="lg" minWidth={900}>
          <Stack direction="column" spacing={1}>
            <ModalHeader caption="edit" title="Edit policy in policy set" />

            <Box padding={2}>
              {error && (
                <Box paddingBottom={2}>
                  <Alert color="danger">
                    <Box>{error.message}</Box>
                  </Alert>
                </Box>
              )}
              <PolicyForm
                initialValues={{
                  ...policy,
                  rules: policy.rules.filter(({ effect }) => effect === "Deny"),
                }}
                isSubmitPending={isPending}
                submitText="Edit policy"
                onSubmit={onSubmit}
                backButton={
                  <Button
                    startDecorator={<ArrowBackIcon />}
                    variant="plain"
                    color="neutral"
                    onClick={() =>
                      navigate({
                        to: "/member/policy_set/$policySetId",
                        params,
                      })
                    }
                  >
                    Back to policy set
                  </Button>
                }
              />
            </Box>
          </Stack>
        </ModalDialog>
      </ModalOverflow>
    </Modal>
  );
}
