import { createFileRoute, useNavigate } from "@tanstack/react-router";
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
import { ModalHeader } from "@/components/modal-header";
import { PolicyForm } from "@/components/policy-form";
import { Policy, useAddPolicyToPolicySet } from "@/network/policy-set";

export const Route = createFileRoute(
  "/__auth/member/policy_set/$policySetId/add_policy",
)({
  component: Component,
});

function Component() {
  const params = Route.useParams();
  const navigate = useNavigate();

  const {
    mutateAsync: addPolicy,
    isPending,
    error,
  } = useAddPolicyToPolicySet({
    policySetId: params.policySetId,
  });

  function onSubmit(policy: Omit<Policy, "id">) {
    addPolicy({
      policy,
    }).then(() => {
      navigate({ to: "/member/policy_set/$policySetId", params });
    });
  }

  return (
    <Modal
      open={true}
      onClose={() =>
        navigate({ to: "/member/policy_set/$policySetId", params })
      }
    >
      <ModalOverflow>
        <ModalDialog sx={{ padding: 0 }} size="lg" minWidth={900}>
          <Stack direction="column" spacing={1}>
            <ModalHeader caption="add" title="Add policy to policy set" />

            <Box padding={2}>
              {error && (
                <Box paddingBottom={2}>
                  <Alert color="danger">
                    <Box>{error.message}</Box>
                  </Alert>
                </Box>
              )}
              <PolicyForm
                isSubmitPending={isPending}
                submitText="Add policy"
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
