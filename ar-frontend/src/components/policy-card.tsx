import { Box, Card, Typography, Stack } from "@mui/joy";
import { Policy } from "../network/policy-set";
import { ReactNode } from "react";
import { arrayValueToDisplay } from "@/array-to-display";

function PolicyCardItem({
  title,
  description,
  paddingLeft,
}: {
  title: string;
  description: string;
  paddingLeft?: number;
}) {
  return (
    <>
      <Box paddingLeft={paddingLeft} display="grid" gridColumn={1}>
        <Typography textColor="neutral.800" level="body-xs">
          {title}
        </Typography>
      </Box>
      <Box display="grid" gridColumn={2} paddingLeft={2}>
        <Typography
          whiteSpace="pre-wrap"
          textColor="primary.500"
          level="body-xs"
        >
          {description}
        </Typography>
      </Box>
    </>
  );
}

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

export function PolicyCard({
  policy,
  actions,
}: {
  policy: Omit<Policy, "id">;
  actions?: ReactNode;
}) {
  return (
    <Card sx={{ backgroundColor: "background.level1" }} size="sm">
      <Box
        display="flex"
        flexDirection="column"
        justifyContent="space-between"
        height="100%"
      >
        <Box>
          <Box display="grid">
            <PolicyCardItem
              title="Actions"
              description={arrayValueToDisplay(policy.actions)}
            />
            <PolicyCardItem
              title="Resource type"
              description={policy.resource_type}
            />
            <PolicyCardItem
              title="Service providers"
              description={arrayValueToDisplay(policy.service_providers)}
            />
            <PolicyCardItem
              title="Attributes"
              description={arrayValueToDisplay(policy.attributes)}
            />
            <PolicyCardItem
              title="Identifiers"
              description={arrayValueToDisplay(policy.identifiers)}
            />
            <Typography textColor="neutral.800" level="body-xs">
              Rules
            </Typography>
          </Box>
          <Stack spacing={1}>
            {policy.rules.map((r, idx) => (
              <Box key={idx} gridColumn={1} display="grid">
                <Card
                  sx={{ backgroundColor: "background.level2", paddingLeft: 2 }}
                >
                  <Box display="grid">
                    <Box gridColumn={1}>
                      <Typography textColor="neutral.800" level="body-xs">
                        Effect
                      </Typography>
                    </Box>
                    <Box gridColumn={2} paddingLeft={2}>
                      <Typography textColor="primary.500" level="body-xs">
                        {r.effect}
                      </Typography>
                    </Box>
                    {r.effect === "Deny" && (
                      <>
                        <PolicyCardItem
                          title="Identifiers"
                          description={arrayValueToDisplay(
                            r.target.resource.identifiers,
                          )}
                        />
                        <PolicyCardItem
                          title="Attributes"
                          description={arrayValueToDisplay(
                            r.target.resource.attributes,
                          )}
                        />
                        <PolicyCardItem
                          title="Actions"
                          description={arrayValueToDisplay(r.target.actions)}
                        />
                        <PolicyCardItem
                          title="Resource type"
                          description={r.target.resource.type}
                        />
                      </>
                    )}
                  </Box>
                </Card>
              </Box>
            ))}
          </Stack>
        </Box>
        {actions && <Box paddingY={1}>{actions}</Box>}
      </Box>
    </Card>
  );
}
