import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/__auth/new_policy_set/step2")({
  component: () => <div>Hello /__auth/new_policy_set/step2!</div>,
});
