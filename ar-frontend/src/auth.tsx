import {
  createContext,
  ReactNode,
  useContext,
  useState,
  useCallback,
} from "react";
import * as jose from "jose";
import { z } from "zod";
import { baseAPIUrl } from "./network/fetch";

type AuthContext = {
  token: string | null;
  setToken: (token: string) => void;
  getToken: () => string;
};










const tokenSchema = z.object({
  exp: z.number(),
         company_id: z.string(),
  realm_access_roles: z.array(z.string()),
  user_id: z.string(),
});

export type Token = z.infer<typeof tokenSchema>;

export const authContext = createContext<AuthContext | null>(null);

export function getTokenContent(token: string): number {
  const decoded = jose.decodeJwt(token);
  const result = tokenSchema.safeParse(decoded);
  if (result.success === false) {
    console.debug(`unable to parse token ${token}`);
    throw new Error("Something unexpected went wrong");
  }

  return result.data;
}

export function isAuthenticated(token: string | null): boolean {
  if (token === null) {
    return false;
  }
  const decoded = getTokenContent(token);

  if (token === null) {
    return false;
  }

  const now = Date.now() / 1000;

  if (decoded.exp - now < 60) {
    return false;
  }

  return true;
}

function initLogin() {
  const redirectUrl = encodeURIComponent(window.location.href);
  const loginUrl = `${baseAPIUrl}/connect/human/auth?redirect_uri=${redirectUrl}`;
  window.location.href = loginUrl;
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(null);

  const getToken = useCallback(() => {
    if (token === null) {
      initLogin();
      throw new Error("initiating login");
    }

    if (!isAuthenticated(token)) {
      initLogin();
      throw new Error("logging in required");
    }

    return token;
  }, [token]);

  return (
    <authContext.Provider value={{ token, getToken, setToken }}>
      {children}
    </authContext.Provider>
  );
}

export function useAuth() {
  const auth = useContext(authContext);

  if (!auth) {
    throw new Error("auth context not initialiased");
  }

  return auth;
}
