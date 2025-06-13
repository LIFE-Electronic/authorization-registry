import { Box, Card, Typography, Stack, Chip, ChipProps } from "@mui/joy";
import { Policy } from "../network/policy-set";
import { ReactNode } from "react";
import { Caption } from "./extra-typography";

export function PolicyCardContainer({ children }: { children: ReactNode }) {
  return (
    <Card
      sx={{ backgroundColor: "background.level1", width: "fit-content" }}
      size="sm"
    >
      <Box display="grid">{children}</Box>
    </Card>
  );
}

function WhiteChip(props: ChipProps) {
  return (
    <Chip
      color="neutral"
      sx={(theme) => ({
        // label: {
        //   whiteSpace: "none",
        // },
        minWidth: "32px",
        textAlign: "center",
        paddingLeft: "8px",
        paddingRight: "8px",
        paddingTop: "6px",
        paddingBottom: "6px",
        height: "24px",
        borderRadius: "36px",
        letterSpacing: "0px",
        lineHeight: "100%",
        borderColor: theme.vars.palette.neutral[500],
        typography: {
          fontWeight: 300,
          lineHeight: "118%",
          letterSpacing: "-0.43%",
          fontSize: "14px",
          color: theme.vars.palette.neutral[600],
        },
      })}
      {...props}
    />
  );
}

function MultiChip({
  values,
  detailed,
}: {
  values: string[];
  detailed: boolean;
}) {
  if (detailed) {
    return (
      <Box display="flex" gap={1} flexWrap="wrap">
        {values.map((value, idx) => (
          <WhiteChip key={idx} size="sm" variant="outlined">
            {value}
          </WhiteChip>
        ))}
      </Box>
    );
  } else {
    return (
      <Box display="flex" gap={1}>
        {values.length > 0 && (
          <WhiteChip size="sm" variant="outlined">
            {values[0]}
          </WhiteChip>
        )}

        {values.length > 1 && (
          <Chip
            sx={(theme) => ({
              typography: {
                fontWeight: 300,
                lineHeight: "118%",
                letterSpacing: "-0.43%",
                fontSize: "14px",
                color: "white",
              },
              backgroundColor: theme.vars.palette.neutral[500],
            })}
            variant="solid"
            color="neutral"
          >
            +{values.length - 1}
          </Chip>
        )}
      </Box>
    );
  }
}

function DetailedRules({ rules }: { rules: Policy["rules"] }) {
  return (
    <Box display="flex" flexDirection="column" gap={1}>
      {rules.map(
        (rule, idx) =>
          rule.effect === "Deny" && (
            <Card key={idx}>
              <Box display="flex" flexDirection="column" gap={1.5} key={idx}>
                <Box display="flex" alignItems="center">
                  <Box width="120px" minWidth="120px">
                    <Caption>Resource type</Caption>
                  </Box>
                  <Typography level="body-lg">
                    {rule.target.resource.type}
                  </Typography>
                </Box>
                <Box display="flex" alignItems="center">
                  <Box width="120px" minWidth="120px">
                    <Caption>Actions</Caption>
                  </Box>
                  <Box display="flex" alignItems="center" gap={1}>
                    {rule.target.actions.map((a) => (
                      <Chip
                        key={a}
                        size="sm"
                        color="success"
                        sx={{
                          typography: {
                            fontWeight: 400,
                            fontSize: "11px",
                            textTransform: "uppercase",
                          },
                          paddingLeft: "8px",
                          paddingRight: "8px",
                          paddingTop: "6px",
                          paddingBottom: "6px",
                          borderRadius: "36px",
                          letterSpacing: "0px",
                          lineHeight: "100%",
                        }}
                      >
                        {a}
                      </Chip>
                    ))}
                  </Box>
                </Box>
                <Box display="flex" alignItems="start">
                  <Box width="120px" minWidth="120px">
                    <Caption>Attributes</Caption>
                  </Box>
                  <Box
                    display="flex"
                    alignItems="center"
                    gap={1}
                    overflow="hidden"
                    flexWrap="wrap"
                  >
                    {rule.target.resource.attributes.map((id) => (
                      <WhiteChip variant="outlined" key={id}>
                        {id}
                      </WhiteChip>
                    ))}
                  </Box>
                </Box>
                <Box display="flex" alignItems="start" textOverflow="hidden">
                  <Box width="120px" minWidth="120px">
                    <Caption>Identifiers</Caption>
                  </Box>
                  <Box
                    display="flex"
                    alignItems="center"
                    gap={1}
                    overflow="hidden"
                    flexWrap="wrap"
                  >
                    {rule.target.resource.identifiers.map((id) => (
                      <WhiteChip variant="outlined" key={id}>
                        {id}
                      </WhiteChip>
                    ))}
                  </Box>
                </Box>
              </Box>
            </Card>
          ),
      )}
    </Box>
  );
}

export function PolicyCard({
  policy,
  actions,
  detailed,
}: {
  policy: Omit<Policy, "id">;
  actions?: ReactNode;
  detailed?: boolean;
}) {
  return (
    <Card sx={{ width: "100%", maxWidth: "calc(100% - 36px)" }}>
      <Box
        display="flex"
        flexDirection="column"
        justifyContent="space-between"
        height="100%"
      >
        <Box>
          <Stack spacing={2}>
            <Box
              display="flex"
              alignItems="center"
              justifyContent="space-between"
            >
              <Box display="flex" flexDirection="column">
                <Caption>Resource type</Caption>
                <Typography level="body-lg">{policy.resource_type}</Typography>
              </Box>
              <Box>{actions}</Box>
            </Box>

            <Box display="flex" flexDirection="column" gap={0.5}>
              <Caption>Actions</Caption>

              <Box display="flex" gap={1}>
                {policy.actions.map((action, idx) => (
                  <Chip
                    key={idx}
                    size="sm"
                    color="success"
                    sx={{
                      typography: {
                        fontWeight: 400,
                        fontSize: "11px",
                        textTransform: "uppercase",
                      },
                      paddingLeft: "8px",
                      paddingRight: "8px",
                      paddingTop: "6px",
                      paddingBottom: "6px",
                      borderRadius: "36px",
                      letterSpacing: "0px",
                      lineHeight: "100%",
                    }}
                  >
                    {action}
                  </Chip>
                ))}
              </Box>
            </Box>

            <Box display="flex" flexDirection="column">
              <Caption>Service providers</Caption>
              <Box display="flex" gap={1}>
                {policy.service_providers.map((action, idx) => (
                  <Typography key={idx} level="body-lg">
                    {action}
                  </Typography>
                ))}
              </Box>
            </Box>

            <Box display="flex" flexDirection="column" gap={0.5}>
              <Caption>Attributes</Caption>
              <MultiChip
                detailed={Boolean(detailed)}
                values={policy.attributes}
              />
            </Box>

            <Box display="flex" flexDirection="column" gap={0.5}>
              <Caption>Identifiers</Caption>
              <MultiChip
                detailed={Boolean(detailed)}
                values={policy.identifiers}
              />
            </Box>

            <Box display="flex" flexDirection="column" gap={0.5}>
              <Caption>Rules</Caption>
              {detailed ? (
                <DetailedRules rules={policy.rules} />
              ) : (
                <MultiChip
                  detailed={Boolean(detailed)}
                  values={policy.rules.map((r) => r.effect)}
                />
              )}
            </Box>
          </Stack>
        </Box>
      </Box>
    </Card>
  );
}
