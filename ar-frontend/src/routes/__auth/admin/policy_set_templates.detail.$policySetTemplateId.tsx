import { CatchBoundary } from "@/components/catch-boundary";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { Caption, Subtitle2 } from "@/components/extra-typography";
import { ModalHeader } from "@/components/modal-header";
import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { PolicyCard } from "@/components/policy-card";
import {
  useAdminDeletePolicySetTemplate,
  usePolicySetTemplate,
} from "@/network/policy-set-templates";
import {
  Box,
  Button,
  Modal,
  ModalDialog,
  ModalOverflow,
  Stack,
  Typography,
} from "@mui/joy";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createFileRoute(
  "/__auth/admin/policy_set_templates/detail/$policySetTemplateId",
)({
  errorComponent: CatchBoundary,
  component: Component,
});

function Component() {
  const { policySetTemplateId } = Route.useParams();
  const navigate = useNavigate();
  const { data, isLoading } = usePolicySetTemplate({
    id: policySetTemplateId,
  });
  const {
    mutateAsync: deletePolicySetTemplate,
    isPending,
    error,
  } = useAdminDeletePolicySetTemplate();
  const [deleteOpen, setDeleteOpen] = useState(false);

  return (
    <Modal
      open={true}
      onClose={() => navigate({ to: "/admin/policy_set_templates" })}
    >
      <ModalOverflow>
        <ModalDialog
          maxWidth="900px"
          size="lg"
          minWidth="900px"
          sx={{ padding: 0 }}
        >
          <PageLoadingFallback isLoading={isLoading}>
            <ConfirmDialog
              onSubmit={() =>
                deletePolicySetTemplate(policySetTemplateId).then(() =>
                  navigate({ to: "/admin/policy_set_templates" }),
                )
              }
              isActionPending={isPending}
              error={error}
              isOpen={deleteOpen}
              title="Delete policy set template"
              isDanger
              onSubmitText="Delete"
              onCancelText="Cancel"
              description="Are you sure you want to delete this policy set template?"
              onClose={() => setDeleteOpen(false)}
            />
            <ModalHeader caption="detail" title="Policy set template" />

            <Stack spacing={2} padding={2}>
              <Stack direction="row" spacing={2}>
                <Box>
                  <Caption>Name</Caption>
                  <Subtitle2>{data?.name}</Subtitle2>
                </Box>
                <Box>
                  <Caption>Policy issuer</Caption>
                  <Subtitle2>
                    {data?.policy_issuer ? data?.policy_issuer : "<empty>"}
                  </Subtitle2>
                </Box>
                <Box>
                  <Caption>Access subject</Caption>
                  <Subtitle2>
                    {data?.access_subject ? data?.access_subject : "<empty>"}
                  </Subtitle2>
                </Box>
              </Stack>

              <Box>
                <Caption>Description</Caption>
                <Typography>{data?.description}</Typography>
              </Box>

              <Box>
                <Caption>Policies</Caption>
                <Stack direction="row" spacing={1}>
                  {data?.policies.map((p, idx) => (
                    <PolicyCard policy={p} key={idx} />
                  ))}
                </Stack>
                {data?.policies && data?.policies.length === 0 && (
                  <Subtitle2>{"<empty>"}</Subtitle2>
                )}
              </Box>
            </Stack>

            <Box padding={2}>
              <Button onClick={() => setDeleteOpen(true)} color="danger">
                Delete
              </Button>
            </Box>
          </PageLoadingFallback>
        </ModalDialog>
      </ModalOverflow>
    </Modal>
  );
}
