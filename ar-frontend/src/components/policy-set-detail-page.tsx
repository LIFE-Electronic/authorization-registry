import {
  Box,
  Button,
  Modal,
  ModalDialog,
  ModalOverflow,
  Stack,
} from "@mui/joy";
import { PageLoadingFallback } from "./page-loading-fallback";
import { PolicySetWithPolicies } from "@/network/policy-set";
import { ConfirmDialog } from "./confirm-dialog";
import { Caption, Subtitle2 } from "./extra-typography";
import { PolicyCard } from "./policy-card";
import { useState } from "react";
import { ErrorResponse } from "@/network/fetch";
import { ModalHeader } from "./modal-header";

function DeletePolicyModal({
  onClose,
  onDeletePolicy,
  error,
  pending,
  isOpen,
}: {
  onClose: () => void;
  onDeletePolicy: () => void;
  error: ErrorResponse | null;
  pending?: boolean;
  isOpen: boolean;
}) {
  return (
    <ConfirmDialog
      error={error}
      isActionPending={Boolean(pending)}
      onSubmitText="Delete"
      onCancelText="Cancel"
      onSubmit={onDeletePolicy}
      isOpen={isOpen}
      onClose={onClose}
      title="Delete policy"
      description="Are you sure you want to delete this policy?"
      isDanger
    />
  );
}

function DeletePolicySetModal({
  onDeletePolicySet,
  error,
  pending,
  isOpen,
  onClose,
}: {
  onDeletePolicySet: () => void;
  error: ErrorResponse | null;
  pending?: boolean;
  isOpen: boolean;
  onClose: () => void;
}) {
  // const navigate = useNavigate();
  // const params = Route.useParams();
  // const search = Route.useSearch();

  // }

  // function onClose() {
  //   navigate({
  //     replace: true,
  //     to: "/admin/policy_set/$policySetId",
  //     params,
  //     search: { ...search, delete_policy_set: undefined },
  //   });
  // }

  return (
    <ConfirmDialog
      error={error}
      isActionPending={Boolean(pending)}
      onSubmitText="Delete"
      onCancelText="Cancel"
      onSubmit={onDeletePolicySet}
      isOpen={isOpen}
      onClose={onClose}
      title="Delete policy"
      description="Are you sure you want to delete this policy?"
      isDanger
    />
  );
}

export function PolicySetDetail({
  policySet,
  isLoading,
  onEdit,
  onModalClose,
  onAddPolicy,
  onDeletePolicySet,
  deletePolicySetPending,
  deletePolicySetError,
  onDeletePolicy,
  deletePolicyError,
  deletePolicyPending,
}: {
  policySet?: PolicySetWithPolicies;
  isLoading?: boolean;
  onEdit: (policyId: string) => void;
  onAddPolicy: () => void;
  onModalClose: () => void;
  onDeletePolicySet: () => void;
  deletePolicySetError: ErrorResponse | null;
  deletePolicySetPending?: boolean;
  onDeletePolicy: (policyId: string) => Promise<void>;
  deletePolicyError: ErrorResponse | null;
  deletePolicyPending: boolean;
}) {
  const [deletePolicyId, setDeletePolicyId] = useState<string | undefined>();
  const [deletePolicySetOpen, setDeletePolicySetOpen] = useState(false);

  return (
    <Modal open={true} onClose={onModalClose}>
      <ModalOverflow>
        <ModalDialog
          maxWidth="900px"
          size="lg"
          minWidth="900px"
          sx={{ padding: 0 }}
        >
          <PageLoadingFallback isLoading={Boolean(isLoading)}>
            {policySet && (
              <>
                {deletePolicyId !== undefined && (
                  <DeletePolicyModal
                    onDeletePolicy={() =>
                      onDeletePolicy(deletePolicyId).then(() => {
                        setDeletePolicyId(undefined);
                      })
                    }
                    onClose={() => setDeletePolicyId(undefined)}
                    error={deletePolicyError}
                    isOpen={Boolean(deletePolicyId)}
                    pending={deletePolicyPending}
                  />
                )}

                <DeletePolicySetModal
                  isOpen={deletePolicySetOpen}
                  onDeletePolicySet={onDeletePolicySet}
                  error={deletePolicySetError}
                  onClose={() => setDeletePolicySetOpen(false)}
                  pending={deletePolicySetPending}
                />

                <Stack>
                  <ModalHeader caption="detail" title="View policy set" />

                  <Box padding={2}>
                    <Stack direction="row" spacing={2}>
                      <Box>
                        <Caption>Policy issuer</Caption>
                        <Subtitle2>{policySet.policy_issuer}</Subtitle2>
                      </Box>
                      <Box>
                        <Caption>Access subject</Caption>
                        <Subtitle2>{policySet.access_subject}</Subtitle2>
                      </Box>
                    </Stack>
                    <Box paddingTop={2}>
                      <Caption>Policies</Caption>
                      <Stack
                        spacing={1}
                        direction="row"
                        flexWrap="wrap"
                        useFlexGap
                      >
                        {policySet.policies.map((p) => (
                          <PolicyCard
                            detailed
                            policy={p}
                            key={p.id}
                            actions={
                              <Stack padding={0} spacing={1} direction="row">
                                <Button
                                  onClick={() => setDeletePolicyId(p.id)}
                                  color="danger"
                                  variant="outlined"
                                >
                                  Delete
                                </Button>
                                <Button
                                  onClick={() => onEdit(p.id)}
                                  variant="outlined"
                                >
                                  Edit
                                </Button>
                              </Stack>
                            }
                          />
                        ))}
                      </Stack>
                      <Box paddingTop={1}>
                        <Button variant="soft" onClick={onAddPolicy}>
                          Add policy
                        </Button>
                      </Box>
                    </Box>
                  </Box>
                  <Stack
                    padding={2}
                    direction="row"
                    spacing={1}
                    sx={(theme) => ({
                      borderTopStyle: "solid",
                      borderColor: theme.vars.palette.neutral[100],
                      borderWidth: "1px",
                    })}
                  >
                    <Button
                      size="lg"
                      color="danger"
                      onClick={() => setDeletePolicySetOpen(true)}
                    >
                      Delete policy set
                    </Button>
                  </Stack>
                </Stack>
              </>
            )}
          </PageLoadingFallback>
        </ModalDialog>
      </ModalOverflow>
    </Modal>
  );
}
