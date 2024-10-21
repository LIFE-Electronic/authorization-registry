import { Box, Card, Typography } from "@mui/joy";
import { Policy } from "../network/policy-set";
import { Fragment, ReactNode } from "react";

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
        <Typography textColor="primary.500" level="body-xs">
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

export function PolicyCard({ policy }: { policy: Omit<Policy, "id"> }) {
  return (
    <PolicyCardContainer>
      <PolicyCardItem title="Actions" description={policy.actions.join(", ")} />
      <PolicyCardItem
        title="Resource type"
        description={policy.resource_type}
      />
      <PolicyCardItem
        title="Service providers"
        description={policy.service_providers.join(", ")}
      />
      <PolicyCardItem
        title="Attributes"
        description={policy.attributes.join(", ")}
      />
      <PolicyCardItem
        title="Identifiers"
        description={policy.identifiers.join(", ")}
      />
      <Typography textColor="neutral.800" level="body-xs">
        Rules
      </Typography>
      {policy.rules.map((r, idx) => (
        <Fragment key={idx}>
          {}
          <Box paddingLeft={2} gridColumn={1}>
            <Typography textColor="neutral.800" level="body-xs">
              Effect
            </Typography>
          </Box>
          <Box paddingLeft={2} gridColumn={2}>
            <Typography textColor="primary.500" level="body-xs">
              {r.effect}
            </Typography>
          </Box>
          {r.effect === "Deny" && (
            <>
              <PolicyCardItem
                paddingLeft={2}
                title="Identifiers"
                description={r.target.resource.identifiers.join(", ")}
              />
              <PolicyCardItem
                paddingLeft={2}
                title="Attributes"
                description={r.target.resource.attributes.join(", ")}
              />
              <PolicyCardItem
                paddingLeft={2}
                title="Actions"
                description={r.target.actions.join(", ")}
              />
              <PolicyCardItem
                paddingLeft={2}
                title="Resource type"
                description={r.target.resource.type}
              />
            </>
          )}
        </Fragment>
      ))}
    </PolicyCardContainer>
  );
}
