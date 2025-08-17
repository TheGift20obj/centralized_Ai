import { createPinia } from 'pinia';
import { createApp } from 'vue';
import './index.scss';
import App from './App.vue';
import { AuthClient } from "@dfinity/auth-client";
import { project_chatgpt_backend } from 'declarations/project_chatgpt_backend/index';
import { Principal } from "@dfinity/principal";

const userStartMessages = [
  "Hi, can you help me with a task?",
  "How does blockchain work?",
  "Write me a thank-you email.",
  "Do you have a gift idea for my mom?",
  "What does a dream about water mean?",
  "What are the latest tech news?",
  "I need a recipe for a quick dinner.",
  "Explain to me what inflation is.",
  "Write me a summary of this book.",
  "How can I improve my CV?",
];

export const loginStatus = {
  loggedIn: false,
  principal: null,
  username: "user",
};

export const login = async () => {
  loginStatus.loggedIn = true;
  loginStatus.principal = Principal.fromText("aaaaa-aa");
  loginStatus.username = await getUserName(loginStatus.principal);
  /*const authClient = await AuthClient.create();

  await authClient.login({
    identityProvider: "https://identity.ic0.app/#authorize",
    onSuccess: async () => {
      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal();
      loginStatus.loggedIn = true;
      loginStatus.principal = principal;
      loginStatus.username = await getUserName(loginStatus.principal);
    },
  });*/
};

export async function chatWithBackend(message) {
  return await project_chatgpt_backend.chat(message);
}

export async function createNewChat(principal, chatId, name) {
  return await project_chatgpt_backend.create_new_chat(principal, chatId, name);
}

export async function addChatMessage(principal, chatId, content, role) {
  return await project_chatgpt_backend.add_chat_message(principal, chatId, content, role);
}

export async function getChatHistory(principal, chatId, msgLen) {
  return await project_chatgpt_backend.get_chat_history(principal, chatId, msgLen);
}

export async function deleteChat(principal, chatId) {
  return await project_chatgpt_backend.delete_chat(principal, chatId);
}

export async function renameChat(principal, chatId, newName) {
  return await project_chatgpt_backend.rename_chat(principal, chatId, newName);
}

export async function listChats(principal) {
  return await project_chatgpt_backend.list_chats(principal);
}

export async function setUserName(principal, username) {
  return await project_chatgpt_backend.set_user_name(principal, username);
}

export async function getUserName(principal) {
  return await project_chatgpt_backend.get_user_name(principal);
}

export async function tryPrompt(principal) {
  return await project_chatgpt_backend.try_increment_user_prompt(principal);
}

export function getRandomUserMessages() {
  const shuffled = [...userStartMessages].sort(() => 0.5 - Math.random());
  return shuffled.slice(0, 3);
}

createApp(App).use(createPinia()).mount('#app');