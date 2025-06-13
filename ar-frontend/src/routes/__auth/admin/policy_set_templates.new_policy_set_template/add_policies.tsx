import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  Button,
  Stack,
  Box,
  IconButton,
  FormLabel,
  FormHelperText,
} from "@mui/joy";
import { useCreatePolicySetTemplateContext } from "../policy_set_templates.new_policy_set_template";
import { PolicyCard } from "@/components/policy-card";
import { NewPolicySetTemplateModalWrapper } from "@/components/new-policy-set-template";
import { useState } from "react";
import { PolicyForm } from "@/components/policy-form";
import AddBoxIcon from "@mui/icons-material/AddBox";
import DeleteIcon from "@mui/icons-material/Delete";

export const Route = createFileRoute(
  "/__auth/admin/policy_set_templates/new_policy_set_template/add_policies",
)({
  component: Component,
});

function Component() {
  const navigate = useNavigate();
  const [addPolicyFormOpen, setAddPolicyFormOpen] = useState(false);

  const { value, changeValue } = useCreatePolicySetTemplateContext();

  return (
    <NewPolicySetTemplateModalWrapper
      step="Add policies"
      onNext={() => {
        navigate({
          to: "/admin/policy_set_templates/new_policy_set_template/review_and_submit",
        });
      }}
      onBack={() => {
        navigate({
          to: "/admin/policy_set_templates/new_policy_set_template/define_policy_set_template",
        });
      }}
    >
      <Stack width="100%" spacing={2}>
        <Box>
          <FormLabel sx={{ fontSize: "16px" }}>Add policies</FormLabel>
          <FormHelperText sx={{ fontSize: "16px" }}>
            Add one or more policies to the policy set template
          </FormHelperText>
        </Box>
        {addPolicyFormOpen ? (
          <Box width="100%">
            <PolicyForm
              onSubmit={(policy) => {
                changeValue((old) => ({
                  ...old,
                  policies: [...old.policies, policy],
                }));

                setAddPolicyFormOpen(false);
              }}
              submitText="Add policy"
              backButton={
                <Button
                  variant="plain"
                  size="sm"
                  color="neutral"
                  onClick={() => setAddPolicyFormOpen(false)}
                >
                  Back to add policies step
                </Button>
              }
            />
          </Box>
        ) : (
          <Stack spacing={2} width="100%">
            <Box>
              <Button
                onClick={() => setAddPolicyFormOpen(true)}
                startDecorator={<AddBoxIcon />}
                variant="outlined"
                size="sm"
              >
                Add policy
              </Button>
            </Box>
            <Box flexWrap="wrap" display="flex" gap={1} paddingBottom={2}>
              {value.policies.map((policy, idx) => (
                <Box
                  key={idx}
                  display="flex"
                  width={{
                    xs: "100%",
                    sm: "47%",
                    md: "32%",
                  }}
                  maxWidth={{
                    xs: "100%",
                    sm: "47%",
                    md: "32%",
                  }}
                >
                  <PolicyCard
                    detailed
                    policy={policy}
                    actions={
                      <Box>
                        <IconButton
                          color="danger"
                          onClick={() => {
                            changeValue((old) => ({
                              ...old,
                              policies: old.policies.filter(
                                (_, currentIdx) => currentIdx !== idx,
                              ),
                            }));
                          }}
                        >
                          <DeleteIcon />
                        </IconButton>
                      </Box>
                    }
                  />
                </Box>
              ))}
            </Box>
          </Stack>
        )}
      </Stack>
    </NewPolicySetTemplateModalWrapper>
  );
}
