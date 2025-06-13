import { Card, Box, Stack } from "@mui/joy";
import { PolicyCard } from "./policy-card";
import { PolicySetWithPolicies } from "@/network/policy-set";
import { Caption, Subtitle2 } from "./extra-typography";

export function PolicySetCard({
  policySet,
}: {
  policySet: PolicySetWithPolicies;
}) {
  return (
    <Card sx={{ width: "100%", boxSizing: "border-box" }}>
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
      <Box>
        <Stack spacing={2} direction="row" flexWrap="wrap" useFlexGap>
          {policySet.policies.map((p) => (
            <Box
              key={p.id}
              width={{
                xs: "100%",
                sm: "47%",
                md: "32%",
              }}
              height="100%"
              display="flex"
            >
              <PolicyCard policy={p} />
            </Box>
          ))}
        </Stack>
      </Box>
    </Card>
  );
}
