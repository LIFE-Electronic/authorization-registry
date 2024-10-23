import { createFileRoute, Outlet } from "@tanstack/react-router";
import {
  createContext,
  Dispatch,
  SetStateAction,
  useContext,
  useState,
} from "react";
import { Policy } from "../../network/policy-set";

export const Route = createFileRoute("/__auth/new_policy_set")({
  component: Component,
});

type CreatePolicySet = {
  access_subject: string;
  policy_issuer: string;
  policies: Omit<Policy, "id">[];
};

const defaultValue: CreatePolicySet = {
  access_subject: "",
  policy_issuer: "",
  policies: [],
};

type Context = {
  value: CreatePolicySet;
  changeValue: Dispatch<SetStateAction<CreatePolicySet>>;
};

const newPolicySetContext = createContext<Context>({
  value: defaultValue,
  changeValue: () => {},
});

export function useCreatePolicySetContext() {
  return useContext(newPolicySetContext);
}

function Component() {
  const [value, setValue] = useState(defaultValue);

  return (
    <newPolicySetContext.Provider
      value={{
        value,
        changeValue: setValue,
      }}
    >
      <Outlet />
    </newPolicySetContext.Provider>
  );
}
