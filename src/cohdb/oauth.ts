import {fetch as tauriFetch, HttpVerb} from '@tauri-apps/api/http';
import {OAuth2Client, OAuth2Fetch, OAuth2Token} from "@badgateway/oauth2-client";
import {getStore} from "../config-store/store";

export const client = new OAuth2Client({
  server: 'http://localhost:3000',
  clientId: 'UACB4t8pVkoBKl4q-Exn8Z6XYIKryrhoneG1smHB4Eg',
  tokenEndpoint: '/oauth/token',
  authorizationEndpoint: '/oauth/authorize',
  fetch: oauthFetch
});

export const authURL = () => client.authorizationCode.getAuthorizeUri({
  redirectUri: 'http://localhost:6969',
  scope: ['read']
});

export const tokenFromRedirect = (redirectURL: string) => client.authorizationCode.getTokenFromCodeRedirect(
  redirectURL,
  { redirectUri: 'http://localhost:6969' }
);

export const cohdbWrapper = new OAuth2Fetch({
  client: client,
  getNewToken(): OAuth2Token | Promise<OAuth2Token | null> | null {
    return null;
  },
  storeToken: async (token) => {
    const store = await getStore();
    store.set('cohdb', token);
  },
  getStoredToken: async () => {
    const store = await getStore();
    return store.get('cohdb') as unknown as Promise<OAuth2Token>;
  }
});

async function oauthFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  const res = await tauriFetch(input.toString(), {
    ...init,
    method: init?.method as HttpVerb,
    body: {
      type: 'Text',
      payload: init?.body
    }
  });

  return new Response(JSON.stringify(res.data) as BodyInit, res);
}
