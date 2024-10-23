import { createFileRoute, Outlet } from "@tanstack/react-router";
import {
  createContext,
  Dispatch,
  SetStateAction,
  useContext,
  useState,
} from "react";
import { Policy } from "../../network/policy-set";
import { Typography } from "@mui/joy";

export const Route = createFileRoute(
  "/__auth/policy_set/$policySetId/add_policy",
)({
  component: Component,
});

const defaultValue: Omit<Policy, "id"> = {
  actions: [],
  resource_type: "",
  identifiers: [],
  attributes: [],
  service_providers: [],
  rules: [],
};

type Context = {
  value: Omit<Policy, "id">;
  changeValue: Dispatch<SetStateAction<Omit<Policy, "id">>>;
};

const policyContext = createContext<Context>({
  value: defaultValue,
  changeValue: () => {},
});

export function useAddPolicyContext() {
  return useContext(policyContext);
}

function Component() {
  const [value, setValue] = useState(defaultValue);

  return (
    <policyContext.Provider
      value={{
        value,
        changeValue: setValue,
      }}
    >
      <Typography level="h3" paddingY={2}>
        Add policy
      </Typography>
      <Outlet />
    </policyContext.Provider>
  );
}
