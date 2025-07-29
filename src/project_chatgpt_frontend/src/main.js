import { createPinia } from 'pinia';
import { createApp } from 'vue';
import './index.scss';
import App from './App.vue';
import { AuthClient } from "@dfinity/auth-client";
import { project_chatgpt_backend } from 'declarations/project_chatgpt_backend/index';
import { Principal } from "@dfinity/principal";

const userStartMessages = [
  "Cześć, możesz mi pomóc z zadaniem?",
  "Jak działa blockchain?",
  "Napisz mi maila z podziękowaniem.",
  "Masz pomysł na prezent dla mamy?",
  "Co oznacza sen o wodzie?",
  "Jakie są nowinki ze świata technologii?",
  "Potrzebuję przepis na szybki obiad.",
  "Wyjaśnij mi, co to jest inflacja.",
  "Napisz mi streszczenie tej książki.",
  "Jak mogę poprawić swoje CV?",
];

export const loginStatus = {
  loggedIn: false,
  principal: null,
  username: "user",
};

export const login = async () => {
  /*loginStatus.loggedIn = true;
  loginStatus.principal = Principal.fromText("aaaaa-aa");
  loginStatus.username = await getUserName(loginStatus.principal);*/
  const authClient = await AuthClient.create();

  await authClient.login({
    identityProvider: "https://identity.ic0.app/#authorize",
    onSuccess: async () => {
      const identity = authClient.getIdentity();
      const principal = identity.getPrincipal();
      loginStatus.loggedIn = true;
      loginStatus.principal = principal;
      loginStatus.username = await getUserName(loginStatus.principal);
    },
  });
};

export async function chatWithBackend(message) {
  return await project_chatgpt_backend.chat(message);
}

export async function createNewChat(principal, chatId, name) {
  return await project_chatgpt_backend.create_new_chat(principal, chatId, name);
}

export async function addChatMessage(principal, chatId, question, answer) {
  return await project_chatgpt_backend.add_chat_message(principal, chatId, question, answer);
}

export async function getChatHistory(principal, chatId) {
  return await project_chatgpt_backend.get_chat_history(principal, chatId);
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