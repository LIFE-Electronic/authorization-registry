import {
  createContext,
  Dispatch,
  ReactNode,
  SetStateAction,
  useContext,
  useState,
} from "react";
import { CreatePolicySetTemplate } from "@/network/policy-set-templates";
import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute(
  "/__auth/admin/policy_set_templates/new_policy_set_template",
)({
  component: Component,
});

function Component() {
  return (
    <CreatePolicySetTemplateContext>
      <Outlet />
    </CreatePolicySetTemplateContext>
  );
}

const defaultValue: CreatePolicySetTemplate = {
  access_subject: "",
  name: "",
  description: "",
  policy_issuer: "",
  policies: [],
};

type Context = {
  value: CreatePolicySetTemplate;
  changeValue: Dispatch<SetStateAction<CreatePolicySetTemplate>>;
};

const newPolicySetContext = createContext<Context>({
  value: defaultValue,
  changeValue: () => {},
});

export function useCreatePolicySetTemplateContext() {
  return useContext(newPolicySetContext);
}

export function CreatePolicySetTemplateContext({
  children,
}: {
  children: ReactNode;
}) {
  const [value, setValue] = useState(defaultValue);

  return (
    <newPolicySetContext.Provider
      value={{
        value,
        changeValue: setValue,
      }}
    >
      {children}
    </newPolicySetContext.Provider>
  );
}
