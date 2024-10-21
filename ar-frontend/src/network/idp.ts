import { baseAPIUrl } from "./fetch";

export const IDP_URL =
  "https://idp.dev.dexes.eu/realms/dmi_dexspace_smartcity/protocol/openid-connect/";

export function initLogin() {
  const redirectUrl = encodeURIComponent(window.location.href);
  const loginUrl = `${baseAPIUrl}/connect/human/auth?redirect_uri=${redirectUrl}`;
  window.location.href = loginUrl;
}

export function initLogout() {
  const redirectUrl = encodeURIComponent(window.location.href);
  const logoutUrl = `${IDP_URL}/logout?post_logout_redirect_url=${redirectUrl}`;
  window.location.href = logoutUrl;
}
