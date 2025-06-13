import { CatchBoundary } from "@/components/catch-boundary";
import { PageLoadingFallback } from "@/components/page-loading-fallback";
import { usePolicySetTemplates } from "@/network/policy-set-templates";
import { Box, Button, Card, Stack, Typography } from "@mui/joy";
import {
  createFileRoute,
  Link,
  Outlet,
  useNavigate,
} from "@tanstack/react-router";

export const Route = createFileRoute("/__auth/admin/policy_set_templates")({
  component: Component,
  errorComponent: CatchBoundary,
});

function Component() {
  const navigate = useNavigate();
  const { data: policyTemplates, isLoading } = usePolicySetTemplates();

  return (
    <>
      <Outlet />
      <PageLoadingFallback isLoading={isLoading}>
        <Box paddingY={4} display="flex" gap={4}>
          <Typography level="h2">Policy set templates</Typography>
          <div>
            <Button
              sx={{
                borderRadius: "8px",
                height: "43px",
                boxShadow: "0px 0px 36px 0px #FF358340",
                backgroundColor: "#007EFF",
              }}
              onClick={() => {
                navigate({
                  to: "/admin/policy_set_templates/new_policy_set_template/define_policy_set_template",
                });
              }}
            >
              New policy set template
            </Button>
          </div>
        </Box>

        <Stack spacing={1} paddingTop={2}>
          {policyTemplates?.map(({ name, id }) => (
            <Link
              key={id}
              to="/admin/policy_set_templates/detail/$policySetTemplateId"
              params={{ policySetTemplateId: id }}
              style={{
                textDecorationLine: "none",
              }}
            >
              <Card>
                <Typography level="title-lg">{name}</Typography>
              </Card>
            </Link>
          ))}
        </Stack>
      </PageLoadingFallback>
    </>
  );
}
