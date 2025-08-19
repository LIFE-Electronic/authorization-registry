import { baseAPIUrl } from "./fetch";

export const IDP_URL = new URL(
  "protocol/openid-connect",
  import.meta.env.VITE_IDP_URL,
);

export function initLogin() {
  async function run() {
    try {
      const baseUrl = `${window.location.protocol}//${window.location.hostname}${window.location.port ? `:${window.location.port}` : ""}`;
      const redirectUrl = `${baseUrl}/callback?state=${`${window.location.pathname}${window.location.search}${window.location.hash}`}`;
      const getParamsUrl = `${baseAPIUrl}/connect/human/auth_params?redirect_uri=${redirectUrl}`;
      const params = await (await fetch(getParamsUrl)).json();

      const loginUrl = `${baseAPIUrl}/connect/human/auth`;

      const form = document.createElement("form");
      form.setAttribute("method", "POST");
      form.setAttribute("action", loginUrl);

      for (const [key, value] of Object.entries(params)) {
        const hiddenField = document.createElement("input");
        hiddenField.setAttribute("type", "hidden");
        hiddenField.setAttribute("name", key);
        hiddenField.setAttribute("value", value as string);

        form.appendChild(hiddenField);
      }

      document.body.appendChild(form);

      form.submit();
    } catch (e) {
      console.error(`error in run: ${e}`);
    }
  }
  return new Promise(run);
}

export function initLogout() {
  const redirectUrl = encodeURIComponent(window.location.href);
  const logoutUrl = `${IDP_URL}/logout?post_logout_redirect_url=${redirectUrl}`;
  window.location.href = logoutUrl;
}
