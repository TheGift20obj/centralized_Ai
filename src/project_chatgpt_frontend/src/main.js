import { createPinia } from 'pinia';
import { createApp } from 'vue';
import './index.scss';
import App from './App.vue';
import { AuthClient } from "@dfinity/auth-client";
import { project_chatgpt_backend } from 'declarations/project_chatgpt_backend/index';

export const loginStatus = {
  loggedIn: false,
  principal: null
};

export const login = async () => {
  const authClient = await AuthClient.create();

  await authClient.login({
    identityProvider: "https://identity.ic0.app/#authorize",
    onSuccess: async () => {
      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal().toText();
      loginStatus.loggedIn = true;
      loginStatus.principal = principal;
    }
  });
};

export async function chatWithBackend(message) {
  return await project_chatgpt_backend.chat(message);
}

createApp(App).use(createPinia()).mount('#app');